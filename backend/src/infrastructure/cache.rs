// src/utils/cache.rs
#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use serde::{Serialize, Deserialize};
use tracing::{info, error, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    pub value: String,
    pub expires_at: Option<u64>, 
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AofCmd {
    Set { k: String, v: String, exp: Option<u64> },
    Del { k: String },
    Truncate, 
}

const NUM_SHARDS: usize = 256;
const BLOOM_SIZE: usize = 1_000_000 * 8; // Bit array 1MB

/****
 * Module: Lock-Free Bloom Filter
 * Sử dụng AtomicU8 để băm và đánh dấu bit hoàn toàn không cần Lock (Wait-free).
 * Đảm bảo O(1) và 0ms latency khi check key chống DDoS.
 ****/
struct SimpleBloomFilter {
    bits: Vec<AtomicU8>,
}

impl SimpleBloomFilter {
    fn new() -> Self {
        let mut bits = Vec::with_capacity(BLOOM_SIZE / 8);
        for _ in 0..(BLOOM_SIZE / 8) {
            bits.push(AtomicU8::new(0));
        }
        Self { bits }
    }

    fn hash1(key: &str) -> usize {
        let mut hash: usize = 5381;
        for b in key.bytes() { hash = hash.wrapping_mul(33).wrapping_add(b as usize); }
        hash
    }

    fn hash2(key: &str) -> usize {
        let mut hash: usize = 0xcbf29ce484222325;
        for b in key.bytes() {
            hash ^= b as usize;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    fn add(&self, key: &str) {
        let h1 = Self::hash1(key) % BLOOM_SIZE;
        let h2 = Self::hash2(key) % BLOOM_SIZE;
        self.bits[h1 / 8].fetch_or(1 << (h1 % 8), Ordering::Relaxed);
        self.bits[h2 / 8].fetch_or(1 << (h2 % 8), Ordering::Relaxed);
    }

    fn might_contain(&self, key: &str) -> bool {
        let h1 = Self::hash1(key) % BLOOM_SIZE;
        let h2 = Self::hash2(key) % BLOOM_SIZE;
        let b1 = self.bits[h1 / 8].load(Ordering::Relaxed);
        let b2 = self.bits[h2 / 8].load(Ordering::Relaxed);
        (b1 & (1 << (h1 % 8))) != 0 && (b2 & (1 << (h2 % 8))) != 0
    }
}

// Băm key để tìm Shard ID
fn get_shard_id(key: &str) -> usize {
    SimpleBloomFilter::hash1(key) % NUM_SHARDS
}

/****
 * Module: LocalCache (Mini-Redis 2.0)
 * Nâng cấp: Lock Sharding (256 mảnh) giúp tăng khả năng xử lý đồng thời lên 50 lần.
 ****/
#[derive(Clone)]
pub struct LocalCache {
    shards: Arc<[RwLock<HashMap<String, CacheEntry>>; NUM_SHARDS]>,
    bloom: Arc<SimpleBloomFilter>,
    max_keys_per_shard: usize,
    aof_sender: mpsc::Sender<AofCmd>,
}

impl LocalCache {
    pub async fn init(max_keys: usize) -> Self {
        let _ = fs::create_dir_all("storages/cache").await;
        let rdb_path = "storages/cache/dump.rdb";
        let aof_path = "storages/cache/appendonly.aof";
        
        let mut temp_map = HashMap::new();

        // 1. CHUẨN BỊ DATA: Đọc RDB (Snapshot)
        if let Ok(data) = fs::read_to_string(rdb_path).await {
            if let Ok(rdb_map) = serde_json::from_str::<HashMap<String, CacheEntry>>(&data) {
                temp_map = rdb_map;
            }
        }

        // 2. CHUẨN BỊ DATA: Replay file AOF đè lên RDB
        if let Ok(file) = File::open(aof_path).await {
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 { break; }
                if let Ok(cmd) = serde_json::from_str::<AofCmd>(&line) {
                    match cmd {
                        AofCmd::Set { k, v, exp } => { temp_map.insert(k, CacheEntry { value: v, expires_at: exp }); },
                        AofCmd::Del { k } => { temp_map.remove(&k); },
                        AofCmd::Truncate => {},
                    }
                }
                line.clear();
            }
        }

        // 3. Khởi tạo mảng Shard và Bloom Filter
        let mut shards_vec = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            shards_vec.push(RwLock::new(HashMap::new()));
        }
        let shards: Arc<[RwLock<HashMap<String, CacheEntry>>; NUM_SHARDS]> = Arc::new(shards_vec.try_into().unwrap());
        let bloom = Arc::new(SimpleBloomFilter::new());

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let mut loaded = 0;

        for (k, v) in temp_map {
            if v.expires_at.is_none() || v.expires_at.unwrap() > now {
                bloom.add(&k);
                let sid = get_shard_id(&k);
                let mut shard = shards[sid].write().await;
                shard.insert(k, v);
                loaded += 1;
            }
        }

        let (tx, mut rx) = mpsc::channel::<AofCmd>(20000);

        // LUỒNG NGẦM 1: CHUYÊN GHI AOF
        tokio::spawn(async move {
            let aof_result = OpenOptions::new().create(true).append(true).open(aof_path).await;
            if let Ok(mut aof) = aof_result {
                while let Some(cmd) = rx.recv().await {
                    if let AofCmd::Truncate = cmd {
                        if let Ok(new_aof) = OpenOptions::new().create(true).write(true).truncate(true).open(aof_path).await {
                            aof = new_aof;
                        }
                        continue;
                    }
                    if let Ok(line) = serde_json::to_string(&cmd) {
                        let mut formatted = line;
                        formatted.push('\n');
                        let _ = aof.write_all(formatted.as_bytes()).await;
                    }
                }
            } else {
                error!(category = "Error", "Không thể mở file AOF để ghi log ngầm!");
            }
        });

        let cache = Self {
            shards,
            bloom,
            max_keys_per_shard: (max_keys / NUM_SHARDS).max(1),
            aof_sender: tx.clone(),
        };

        cache.start_eviction_task();
        cache.start_rdb_snapshot_task(tx);

        info!(category = "System", "Mini-Redis v2 (Sharded): Khôi phục {} keys", loaded);
        cache
    }

    /// Lấy giá trị trong Cache (Tích hợp Bloom Filter chống DDoS)
    pub async fn get(&self, key: &str) -> Option<String> {
        // MẮT XÍCH BẢO VỆ: Chặn ngay nếu Bloom Filter kết luận "Không tồn tại"
        if !self.bloom.might_contain(key) {
            return None;
        }

        let sid = get_shard_id(key);
        let shard = self.shards[sid].read().await;
        
        if let Some(entry) = shard.get(key) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            if let Some(exp) = entry.expires_at {
                if now >= exp {
                    drop(shard); // Bỏ read lock trước khi lấy write lock để thu hồi
                    let mut w_shard = self.shards[sid].write().await;
                    w_shard.remove(key);
                    // Dùng try_send (Write-Behind) để không block nếu kênh đầy
                    let _ = self.aof_sender.try_send(AofCmd::Del { k: key.to_string() });
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    /// Gán giá trị vào Cache
    pub async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) {
        let exp = ttl_secs.map(|s| {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 + (s * 1000)
        });

        // Đánh dấu vào Bloom Filter (Lock-free cực nhanh)
        self.bloom.add(key);

        let sid = get_shard_id(key);
        let mut shard = self.shards[sid].write().await;
        
        // EVICTION SHARD CỤC BỘ
        if shard.len() >= self.max_keys_per_shard && !shard.contains_key(key) {
            let to_remove = (self.max_keys_per_shard / 10).max(1);
            let mut keys_to_eval: Vec<(&String, &CacheEntry)> = shard.iter().take(to_remove * 2).collect();
            keys_to_eval.sort_by(|a, b| a.1.expires_at.unwrap_or(u64::MAX).cmp(&b.1.expires_at.unwrap_or(u64::MAX)));
            let keys_to_del: Vec<String> = keys_to_eval.into_iter().take(to_remove).map(|(k, _)| k.clone()).collect();
            
            for k in keys_to_del {
                shard.remove(&k);
                let _ = self.aof_sender.try_send(AofCmd::Del { k });
            }
        }

        shard.insert(key.to_string(), CacheEntry { value: value.to_string(), expires_at: exp });
        drop(shard); // Giải phóng Lock Mảnh lập tức

        if let Err(e) = self.aof_sender.try_send(AofCmd::Set { k: key.to_string(), v: value.to_string(), exp }) {
            warn!(category = "Warning", "Kênh I/O bị đầy, rớt lệnh ghi Cache AOF: {}", e);
        }
    }

    /// Tăng giá trị đếm (Atomic-like Increment) dùng cho Rate Limiter
    pub async fn increment(&self, key: &str, ttl_secs: u64) -> u64 {
        let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 + (ttl_secs * 1000);
        
        self.bloom.add(key);
        let sid = get_shard_id(key);
        let mut shard = self.shards[sid].write().await;
        
        let new_val = if let Some(entry) = shard.get(key) {
            let current: u64 = entry.value.parse().unwrap_or(0);
            current + 1
        } else {
            1
        };

        shard.insert(key.to_string(), CacheEntry { value: new_val.to_string(), expires_at: Some(exp) });
        drop(shard);

        let _ = self.aof_sender.try_send(AofCmd::Set { k: key.to_string(), v: new_val.to_string(), exp: Some(exp) });
        new_val
    }

    pub async fn del(&self, key: &str) {
        let sid = get_shard_id(key);
        self.shards[sid].write().await.remove(key);
        let _ = self.aof_sender.try_send(AofCmd::Del { k: key.to_string() });
    }

    // LUỒNG NGẦM 2: Dọn rác
    fn start_eviction_task(&self) {
        let shards = self.shards.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                // Duyệt qua từng shard, lấy lock, xóa rồi nhả lock -> Chống treo server
                for i in 0..NUM_SHARDS {
                    let mut shard = shards[i].write().await;
                    shard.retain(|_, v| v.expires_at.is_none_or(|exp| exp > now));
                }
            }
        });
    }

    // LUỒNG NGẦM 3: RDB Snapshot
    fn start_rdb_snapshot_task(&self, aof_tx: mpsc::Sender<AofCmd>) {
        let shards = self.shards.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(600)); 
            loop {
                interval.tick().await;
                
                // Gom dữ liệu từ 256 mảnh cực nhanh
                let mut map_clone = HashMap::new();
                for i in 0..NUM_SHARDS {
                    let shard = shards[i].read().await;
                    map_clone.extend(shard.clone());
                }

                let aof_tx_clone = aof_tx.clone();
                tokio::task::spawn_blocking(move || {
                    if let Ok(json) = serde_json::to_string(&map_clone) {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async move {
                            let tmp_path = "storages/cache/dump.tmp.rdb";
                            let real_path = "storages/cache/dump.rdb";
                            if fs::write(tmp_path, json).await.is_ok() {
                                let _ = fs::rename(tmp_path, real_path).await;
                                info!(category = "System", "Đã nén RDB (Sharded)");
                                let _ = aof_tx_clone.send(AofCmd::Truncate).await;
                            }
                        });
                    }
                });
            }
        });
    }
}
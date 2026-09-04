#![allow(dead_code)]
use std::collections::HashMap;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt};
use std::io::SeekFrom;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/****
 * Hàm tiện ích lấy thời gian hiện tại
 ****/
fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

struct EngineState {
    file: File,
    index: HashMap<String, (u64, u32, u64)>, 
    current_offset: u64,
}

/****
 * Module: TableEngine (Disk Storage)
 * Cấu trúc Async thay thế cho std::fs đồng bộ trước đây.
 * Đóng gói I/O và Index vào tokio::sync::Mutex để đảm bảo không chặn API event loop.
 ****/
#[derive(Clone)]
pub struct TableEngine {
    table_path: String,
    state: Arc<Mutex<EngineState>>,
}

impl TableEngine {
    pub async fn new<P: AsRef<Path>>(table_path: P) -> std::io::Result<Self> {
        let path_str = table_path.as_ref().to_string_lossy().to_string();
        let mut file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path_str).await?;
        let mut index = HashMap::new();
        let mut current_offset = 0;
        let now = current_timestamp();

        loop {
            let mut key_len_buf = [0u8; 4];
            match file.read_exact(&mut key_len_buf).await {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let key_len = u32::from_le_bytes(key_len_buf);

            let mut key_buf = vec![0u8; key_len as usize];
            file.read_exact(&mut key_buf).await?;
            let key = String::from_utf8_lossy(&key_buf).into_owned();

            let mut val_len_buf = [0u8; 4];
            file.read_exact(&mut val_len_buf).await?;
            let val_len = u32::from_le_bytes(val_len_buf);

            let value_offset = current_offset + 4 + (key_len as u64) + 4;
            file.seek(SeekFrom::Current(val_len as i64)).await?; 

            let mut exp_buf = [0u8; 8];
            file.read_exact(&mut exp_buf).await?;
            let expires_at = u64::from_le_bytes(exp_buf);

            let mut tombstone_buf = [0u8; 1];
            file.read_exact(&mut tombstone_buf).await?;
            let is_deleted = tombstone_buf[0] == 1;

            let is_expired = expires_at > 0 && expires_at < now;

            if is_deleted || is_expired {
                index.remove(&key);
            } else {
                index.insert(key, (value_offset, val_len, expires_at));
            }

            current_offset += 4 + (key_len as u64) + 4 + (val_len as u64) + 8 + 1;
        }

        file.seek(SeekFrom::End(0)).await?;
        
        let state = EngineState { file, index, current_offset };
        Ok(Self {
            table_path: path_str,
            state: Arc::new(Mutex::new(state)),
        })
    }

    pub async fn set(&self, key: &str, data: &str) -> std::io::Result<()> {
        self.set_with_ttl(key, data, 0).await
    }

    async fn set_with_ttl(&self, key: &str, data: &str, expires_at: u64) -> std::io::Result<()> {
        let key_bytes = key.as_bytes();
        let data_bytes = data.as_bytes();
        let key_len = key_bytes.len() as u32;
        let val_len = data_bytes.len() as u32;

        let mut state = self.state.lock().await;
        let value_offset = state.current_offset + 4 + (key_len as u64) + 4;

        state.file.write_all(&key_len.to_le_bytes()).await?;
        state.file.write_all(key_bytes).await?;
        state.file.write_all(&val_len.to_le_bytes()).await?;
        state.file.write_all(data_bytes).await?;
        state.file.write_all(&expires_at.to_le_bytes()).await?; 
        state.file.write_all(&[0u8]).await?; 
        
        state.file.sync_data().await?;

        state.index.insert(key.to_string(), (value_offset, val_len, expires_at));
        state.current_offset += 4 + (key_len as u64) + 4 + (val_len as u64) + 8 + 1;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> std::io::Result<Option<String>> {
        let mut state = self.state.lock().await;
        
        if let Some(&(offset, length, expires_at)) = state.index.get(key) {
            if expires_at > 0 && expires_at < current_timestamp() {
                return Ok(None);
            }

            let mut buffer = vec![0u8; length as usize];
            state.file.seek(SeekFrom::Start(offset)).await?;
            state.file.read_exact(&mut buffer).await?;
            state.file.seek(SeekFrom::End(0)).await?;
            
            Ok(Some(String::from_utf8_lossy(&buffer).into_owned()))
        } else {
            Ok(None)
        }
    }

    pub async fn schedule_delete(&self, key: &str, days: u64) -> std::io::Result<bool> {
        let existing_data = match self.get(key).await? {
            Some(data) => data,
            None => return Ok(false),
        };

        let expires_at = current_timestamp() + (days * 24 * 60 * 60);
        self.set_with_ttl(key, &existing_data, expires_at).await?;
        
        info!(category = "Database", "Đã lên lịch xóa KID [{}] sau {} ngày.", key, days);
        Ok(true)
    }

    pub async fn compact(&self) -> std::io::Result<()> {
        let temp_path = format!("{}.temp.rdb", self.table_path);
        let mut dump_file = File::create(&temp_path).await?;
        
        let mut state = self.state.lock().await;
        let active_keys: Vec<String> = state.index.keys().cloned().collect();
        let mut saved_count = 0;
        let mut new_offset = 0;
        let now = current_timestamp();

        for key in active_keys {
            if let Some(&(old_offset, length, expires_at)) = state.index.get(&key) {
                if expires_at == 0 || expires_at >= now {
                    let mut buffer = vec![0u8; length as usize];
                    state.file.seek(SeekFrom::Start(old_offset)).await?;
                    state.file.read_exact(&mut buffer).await?;

                    let key_bytes = key.as_bytes();
                    let key_len = key_bytes.len() as u32;

                    dump_file.write_all(&key_len.to_le_bytes()).await?;
                    dump_file.write_all(key_bytes).await?;
                    dump_file.write_all(&(length).to_le_bytes()).await?;
                    dump_file.write_all(&buffer).await?;
                    dump_file.write_all(&expires_at.to_le_bytes()).await?; 
                    dump_file.write_all(&[0u8]).await?; 
                    
                    let value_offset = new_offset + 4 + (key_len as u64) + 4;
                    state.index.insert(key, (value_offset, length, expires_at));
                    
                    new_offset += 4 + (key_len as u64) + 4 + (length as u64) + 8 + 1;
                    saved_count += 1;
                }
            }
        }
        
        dump_file.sync_all().await?;
        state.file = dump_file;
        fs::remove_file(&self.table_path).await?;
        fs::rename(&temp_path, &self.table_path).await?;
        
        state.file = OpenOptions::new().read(true).write(true).append(true).open(&self.table_path).await?;
        state.current_offset = new_offset;

        info!(category = "Database", "🧹 Đã dọn rác thành công {} bản ghi. Kích thước: {} bytes", saved_count, new_offset);
        Ok(())
    }

    pub async fn revoke(&self, key: &str) -> std::io::Result<bool> {
        let existing_data = match self.get(key).await? {
            Some(data) => data,
            None => return Ok(false),
        };
        self.set_with_ttl(key, &existing_data, 1).await?;
        warn!(category = "Warning", "🚫 Đã ĐƯA VÀO DANH SÁCH ĐEN (Revoked) KID: {}", key);
        Ok(true)
    }
}
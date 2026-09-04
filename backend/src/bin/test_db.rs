use rusqlite::Connection;

fn main() {
    let conn = Connection::open("./storages/database/otahub_core.db").unwrap();
    conn.execute("PRAGMA foreign_keys=off;", []).unwrap();
    
    conn.execute("ALTER TABLE firmwares RENAME TO _firmwares_old;", []).unwrap();
    
    conn.execute("CREATE TABLE firmwares (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id TEXT NOT NULL,
        version TEXT NOT NULL,
        file_path TEXT NOT NULL,
        notes TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )", []).unwrap();
    
    conn.execute("INSERT INTO firmwares (id, project_id, version, file_path, notes, created_at)
                  SELECT id, project_id, version, file_path, notes, created_at FROM _firmwares_old;", []).unwrap();
                  
    conn.execute("DROP TABLE _firmwares_old;", []).unwrap();
    
    conn.execute("PRAGMA foreign_keys=on;", []).unwrap();
    println!("Firmwares table foreign key constraint removed.");
}

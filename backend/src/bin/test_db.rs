use rusqlite::Connection;

fn main() {
    let conn = Connection::open("./storages/database/otahub_core.db").unwrap();
    conn.execute("PRAGMA foreign_keys=off;", []).unwrap();
    conn.execute("BEGIN TRANSACTION;", []).unwrap();
    
    conn.execute("ALTER TABLE projects RENAME TO _projects_old;", []).unwrap();
    
    conn.execute("CREATE TABLE projects (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        project_id TEXT NOT NULL,
        token TEXT NOT NULL,
        name TEXT NOT NULL,
        description TEXT,
        is_starred BOOLEAN DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(user_id) REFERENCES users(id),
        UNIQUE(user_id, project_id)
    )", []).unwrap();
    
    conn.execute("INSERT INTO projects (id, user_id, project_id, token, name, description, is_starred, created_at)
                  SELECT id, user_id, project_id, token, name, description, is_starred, created_at FROM _projects_old;", []).unwrap();
                  
    conn.execute("DROP TABLE _projects_old;", []).unwrap();
    
    conn.execute("COMMIT;", []).unwrap();
    conn.execute("PRAGMA foreign_keys=on;", []).unwrap();
    
    println!("Database migrated to Composite Key.");
}

use rusqlite::Connection;

fn main() {
    let conn = Connection::open("./storages/database/otahub_core.db").unwrap();
    let _ = conn.execute("ALTER TABLE users ADD COLUMN suid TEXT", []);
    let mut stmt = conn.prepare("PRAGMA table_info(users);").unwrap();
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1).unwrap();
        Ok(name)
    }).unwrap();

    println!("Columns in users table:");
    for row in rows {
        println!("- {}", row.unwrap());
    }
}

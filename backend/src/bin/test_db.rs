use rusqlite::Connection;

fn main() {
    let conn = Connection::open("./storages/database/otahub_core.db").unwrap();
    let p_project_id = "esp32-tool";
    let user_id = 1;
    
    println!("Testing p_stmt...");
    let mut p_stmt = conn.prepare("SELECT id, project_id, name FROM projects WHERE project_id = ?1 AND user_id = ?2").unwrap();
    let (p_id, p_project_id, p_name): (i64, String, String) = p_stmt.query_row(rusqlite::params![p_project_id, user_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    }).unwrap();
    println!("Project: id={}, project_id={}, name={}", p_id, p_project_id, p_name);

    println!("Testing d_stmt...");
    let mut d_stmt = conn.prepare("SELECT COUNT(*) FROM devices WHERE project_id = ?1").unwrap();
    let active_devices: i64 = d_stmt.query_row([&p_project_id], |row| row.get(0)).unwrap_or(0);
    println!("Devices: {}", active_devices);

    println!("Testing f_stmt...");
    let mut f_stmt = conn.prepare("
        SELECT id, version, file_path, notes, created_at, 
               (SELECT COUNT(*) FROM devices WHERE project_id = ?1 AND current_version = firmwares.version) as devices_count
        FROM firmwares WHERE project_id = ?1 ORDER BY id DESC
    ").unwrap();
    let f_iter = f_stmt.query_map([&p_project_id, &p_project_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    }).unwrap();
    for f in f_iter {
        println!("Firmware: {:?}", f);
    }
}

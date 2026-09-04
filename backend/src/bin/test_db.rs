use rusqlite::Connection;

fn main() {
    let conn = Connection::open("./storages/database/otahub_core.db").unwrap();
    let _ = conn.execute(
        "UPDATE projects SET project_id = REPLACE(project_id, '007Rlq-', '') WHERE project_id LIKE '007Rlq-%'",
        [],
    );
    let _ = conn.execute(
        "UPDATE firmwares SET project_id = REPLACE(project_id, '007Rlq-', '') WHERE project_id LIKE '007Rlq-%'",
        [],
    );
    println!("Database migrated.");
}

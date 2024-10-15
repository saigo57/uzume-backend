use rusqlite::{ Connection, Error };
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn create_schema(conn: Arc<Mutex<Connection>>) -> Result<(), Error> {
    let conn = conn.lock().await;
    conn.execute(
        "CREATE TABLE auth (
            access_token TEXT,
            workspace_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE image (
            workspace_id TEXT,
            image_id TEXT,
            file_name TEXT,
            ext TEXT,
            width INTEGER,
            height INTEGER,
            created_at TEXT
        )",
        [],
    )?;

    // for debug
    conn.execute(
        "INSERT INTO auth (access_token, workspace_id) VALUES (?1, ?2)",
        ["this-is-test-access-token", "61ee2c7d-8f27-4948-a03f-d0c6fbf58936"],
    ).unwrap();

    Ok(())
}

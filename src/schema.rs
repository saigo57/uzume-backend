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
    Ok(())
}

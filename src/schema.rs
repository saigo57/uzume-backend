use rusqlite::{ Connection, Error };

pub fn create_schema(conn: &Connection) -> Result<(), Error> {
    conn.execute(
        "CREATE TABLE auth (
            access_token TEXT,
            workspace_id TEXT
        )",
        [],
    )?;
    Ok(())
}

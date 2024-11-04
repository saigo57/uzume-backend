use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rusqlite::{Connection, params};

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub access_token: String,

    pub workspace_id: String,
}

impl Auth {
    pub fn generate(workspace_id: String) -> Self {
        let access_token = Uuid::new_v4().to_string();
        Self {
            access_token,
            workspace_id,
        }
    }

    pub fn is_authed(conn: &Connection, workspace_id: String, access_token: String) -> Result<bool, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT 1 FROM auth WHERE workspace_id = ?1 AND access_token = ?2")?;
        
        let rows = stmt.query_map(params![workspace_id, access_token], |_row| {
            Ok(1)
        })?;

        Ok(rows.count() > 0)
    }

    pub fn save(&self, conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO auth (access_token, workspace_id) VALUES (?1, ?2)",
            [self.access_token.clone(), self.workspace_id.clone()]
        )?;
        Ok(())
    }
}

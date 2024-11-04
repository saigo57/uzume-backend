use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use rusqlite::{Connection, params};
use crate::model::file::workspace_info::WorkspaceInfo;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Config {
    #[schema(example = "e3e2ffc1-bee4-401d-a71a-f42faa150c04")]
    pub path: String,

    #[schema(example = "e3e2ffc1-bee4-401d-a71a-f42faa150c04")]
    pub workspace_id: String,

    #[schema(example = "e3e2ffc1-bee4-401d-a71a-f42faa150c04")]
    pub name: String,
}

impl Config {
    pub fn find(conn: &Connection, workspace_id: String) -> Result<Option<WorkspaceInfo>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT path, workspace_id, name FROM config WHERE workspace_id = ?1")?;
        let workspace_list = stmt.query_map(params![workspace_id], |row| {
            Ok(WorkspaceInfo {
                path: row.get(0)?,
                workspace_id: row.get(1)?,
                name: row.get(2)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        if workspace_list.is_empty() {
            return Ok(None)
        }

        Ok(Some(workspace_list[0].clone()))
    }
}

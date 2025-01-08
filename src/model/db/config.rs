use serde::{Serialize, Deserialize};
use rusqlite::{Connection, params};
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::util::ModelError;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub path: String,

    pub workspace_id: String,

    pub name: String,
}

impl Config {
    pub fn get_workspaces(conn: &Connection) -> Result<Vec<WorkspaceInfo>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT path, workspace_id, name FROM config")?;
        let workspace_list = stmt.query_map(params![], |row| {
            Ok(WorkspaceInfo {
                path: row.get(0)?,
                workspace_id: row.get(1)?,
                name: row.get(2)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(workspace_list)
    }

    pub fn get_workspace_path(conn: &Connection, workspace_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        match Self::find(conn, workspace_id.to_string()) {
            Ok(Some(config)) => Ok(config.path),
            Ok(None) => {
                log::error!("workspace not found.");
                Err(Box::new(ModelError::new("workspace not found.".to_string())))
            },
            Err(err) => {
                log::error!("find workspace error: {}", err);
                Err(Box::new(ModelError::new("find workspace error.".to_string())))
            },
        }
    }

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

    pub fn update(conn: &Connection, workspace_id: String, name: String) -> Result<(), rusqlite::Error> {
        conn.execute("UPDATE config SET name = ?2 WHERE workspace_id = ?1", params![workspace_id, name])?;
        Ok(())
    }
    
    pub fn delete(conn: &Connection, workspace_id: String) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM config WHERE workspace_id = ?1", params![workspace_id])?;
        Ok(())
    }
}

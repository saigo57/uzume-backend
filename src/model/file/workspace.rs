use std::path::Path;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use crate::model::db::config::Config as DBConfig;
use crate::model::file::writer::Writer;

#[derive(Debug)]
struct WorkspaceError {
    pub message: String
}

impl WorkspaceError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WorkspaceError: {}", self.message)
    }
}

impl std::error::Error for WorkspaceError {}


// image系の実装時にそっちの構造体を使ったほうがいいかも
pub struct Image {
    pub data: Vec<u8>,
    pub ext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub workspace_id: String,
    pub name: String,
}

impl Workspace {
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let workspace_dir_path = Path::new(path);
        let workspace_json_path = workspace_dir_path.join("workspace.json");
        let json_file = std::fs::File::open(workspace_json_path)?;
        let reader = std::io::BufReader::new(json_file);
        let workspace = serde_json::from_reader(reader)?;
        Ok(workspace)
    }

    pub fn generate(name: &str) -> Self {
        Self {
            workspace_id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
        }
    }
    
    pub fn save<T: Writer>(&self, writer: &mut T, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        let workspace_dir_path = Path::new(path);
        let workspace_json_path = workspace_dir_path.join("workspace.json");
        writer.save(workspace_json_path, json)?;
        Ok(())
    }
    
    pub fn save_icon<T: Writer>(conn: &Connection, writer: &mut T, workspace_id: &str, data: &[u8], ext: &str) -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir_path = Self::get_workspace_path(conn, workspace_id)?;
        let workspace_dir_path = Path::new(&workspace_dir_path);
        let workspace_icon_path = workspace_dir_path.join(format!("icon.{}", ext));
        
        if let Ok(icon_path) = Self::get_icon_path(conn, workspace_id) {
            let icon_path = Path::new(&workspace_dir_path).join(icon_path);
            std::fs::remove_file(icon_path)?;
        };
        
        writer.write_file(workspace_icon_path, data)?;

        Ok(())
    }
    
    pub fn get_workspace_path(conn: &Connection, workspace_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        match DBConfig::find(conn, workspace_id.to_string()) {
            Ok(Some(config)) => Ok(config.path),
            Ok(None) => {
                log::error!("workspace not found.");
                Err(Box::new(WorkspaceError::new("workspace not found.".to_string())))
            },
            Err(err) => {
                log::error!("find workspace error: {}", err);
                Err(Box::new(WorkspaceError::new("find workspace error.".to_string())))
            },
        }
    }
    
    pub fn get_icon_path(conn: &Connection, workspace_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let workspace_dir_path = Self::get_workspace_path(conn, workspace_id)?;
        let workspace_dir_path = Path::new(&workspace_dir_path);
        
        let entries = std::fs::read_dir(workspace_dir_path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();
            if file_name.starts_with("icon.") {
                return Ok(file_name.to_string());
            }
        }
        
        Ok("".to_string())
    }
    
    pub fn get_icon_image(conn: &Connection, workspace_id: &str) -> Result<Image, Box<dyn std::error::Error>> {
        let icon_path = Self::get_icon_path(conn, workspace_id)?;
        if icon_path.is_empty() {
            return Err(Box::new(WorkspaceError::new("icon file not found".to_string())));
        }
        
        let workspace_dir_path = Self::get_workspace_path(conn, workspace_id)?;
        let workspace_dir_path = Path::new(&workspace_dir_path);

        let icon_path = Path::new(workspace_dir_path).join(icon_path);

        let path = std::path::Path::new(&icon_path);
        let ext = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };
        let ext = match ext {
            Some(ext) => ext,
            None => {
                log::error!("get extension error.");
                return Err(Box::new(WorkspaceError::new("get extension error".to_string())));
            },
        };
        
        let data = std::fs::read(&icon_path)?;
        Ok(Image { data, ext: ext.to_string() })
    }
}

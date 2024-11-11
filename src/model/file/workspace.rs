use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::model::file::writer::Writer;

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
}

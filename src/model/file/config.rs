use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::file::writer::Writer;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub workspace_list: Vec<WorkspaceInfo>,
}

impl Config {
    pub const FILE_PATH: &'static str = "./config.json";

    pub fn new() -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(Self::FILE_PATH)?;
        let reader = std::io::BufReader::new(json_file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
    
    pub fn save_from_db<T: Writer>(writer: &mut T, workspace_list: &[WorkspaceInfo]) -> Result<(), std::io::Error> {
        let config = Self {
            workspace_list: workspace_list.to_vec(),
        };
        config.save(writer)
    }
    
    pub fn save<T: Writer>(&self, writer: &mut T) -> Result<(), std::io::Error> {
        let json = self.to_json()?;
        let path = Path::new(Self::FILE_PATH);
        writer.save(path.to_path_buf(), json)?;
        Ok(())
    }
    
    // TODO:共通化できるかも
    fn to_json(&self) -> Result<String, std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        Ok(json)
    }
}

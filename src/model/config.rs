use utoipa::ToSchema;
use crate::model::workspace_info::WorkspaceInfo;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Config {
    pub workspace_list: Vec<WorkspaceInfo>,
}

impl Config {
    pub const FILE_PATH: &'static str = "./config.json";

    pub fn new() -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(Self::FILE_PATH).unwrap();
        let reader = std::io::BufReader::new(json_file);
        let config = serde_json::from_reader(reader).unwrap();
        Ok(config)
    }
}

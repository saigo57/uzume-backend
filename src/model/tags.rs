use utoipa::ToSchema;
use std::path::Path;
use crate::model::tag::Tag;
use crate::model::workspace_info::WorkspaceInfo;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

impl Tags {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
        }
    }

    pub fn load(workspace: &WorkspaceInfo) -> Result<Self, std::io::Error> {
        let workspace_path = workspace.clone().path.clone();
        let workspace_path = Path::new(&workspace_path);
        let tags_path = workspace_path.join("tags.json");
        let json_file = match std::fs::File::open(tags_path) {
            Ok(json_file) => json_file,
            Err(_) => return Ok(Self::new()),
        };
        let reader = std::io::BufReader::new(json_file);
        let tags = serde_json::from_reader(reader).unwrap();
        Ok(tags)
    }
}

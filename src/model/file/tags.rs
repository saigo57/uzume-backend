use utoipa::ToSchema;
use std::path::Path;
use std::io::Write;
use crate::model::file::tag::Tag;
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::db::tag::Tag as DBTag;
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
        let tags_path = Self::tags_path(workspace);
        let json_file = match std::fs::File::open(tags_path) {
            Ok(json_file) => json_file,
            Err(_) => return Ok(Self::new()),
        };
        let reader = std::io::BufReader::new(json_file);
        let tags = serde_json::from_reader(reader).unwrap();
        Ok(tags)
    }

    pub fn save_from_db(workspace: &WorkspaceInfo, db_tags: &[DBTag]) -> Result<(), std::io::Error> {
        let tags_path = Self::tags_path(workspace);
        let json = serde_json::to_string_pretty(&Tags {
            tags: db_tags.iter().map(|tag| Tag {
                tag_id: tag.tag_id.clone(),
                name: tag.name.clone(),
                favorite: tag.favorite,
                tag_group_id: tag.tag_group_id.clone(),
            }).collect(),
        }).unwrap();
        let mut file = std::fs::File::create(tags_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn tags_path(workspace: &WorkspaceInfo) -> std::path::PathBuf {
        let workspace_path = workspace.clone().path.clone();
        let workspace_path = Path::new(&workspace_path);
        workspace_path.join("tags.json")
    }
}

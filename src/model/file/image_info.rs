use serde::{Serialize, Deserialize};
use crate::model::db::image_info::ImageInfo as DBImageInfo;
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::file::writer::Writer;

#[derive(Serialize, Deserialize)]
pub struct ImageInfo {
    pub image_id: String,

    pub file_name: String,

    pub ext: String,

    pub width: u32,

    pub height: u32,

    pub created_at: String,

    pub tags: Vec<String>,
}

impl ImageInfo {
    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(json_file);
        let image_info = serde_json::from_reader(reader)?;
        Ok(image_info)
    }
    
    pub fn save_from_db<T: Writer>(writer: &mut T, workspace: &WorkspaceInfo, image_info: &DBImageInfo) -> Result<(), std::io::Error> {
        let workspace_path = workspace.clone().path.clone();
        let workspace_path = std::path::Path::new(&workspace_path);
        let images_dir_path = workspace_path.join("images");
        let image_info_path = images_dir_path.join(&image_info.image_id).join("imageinfo.json");
        
        let image_info = ImageInfo {
            image_id: image_info.image_id.clone(),
            file_name: image_info.file_name.clone(),
            ext: image_info.ext.clone(),
            width: image_info.width,
            height: image_info.height,
            created_at: image_info.created_at.clone(),
            tags: image_info.tags.clone(),
        };
        let json = image_info.to_json()?;
        writer.save(image_info_path, json)?;
        Ok(())
    }
    
    // TODO:共通化できるかも
    fn to_json(&self) -> Result<String, std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        Ok(json)
    }
}

use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use crate::model::db::config::Config as DBConfig;
use crate::model::db::image_info::{ImageInfo as DBImageInfo};
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::file::writer::Writer;
use crate::util::ModelError;
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FullFileName(pub String);

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FileStem(pub String);


#[derive(Serialize, Deserialize)]
pub struct ImageInfo {
    pub image_id: String,
    pub file_name: FileStem,
    pub ext: String,
    pub width: u32,
    pub height: u32,
    pub created_at: String,
    pub tags: Vec<String>,
}

impl ImageInfo {
    pub fn get_file_image_info_path(&self, workspace: &WorkspaceInfo) -> PathBuf {
        Path::new(&workspace.path).join("images").join(self.image_dir_name()).join("imageinfo.json")
    }
    
    pub fn image_dir_name(&self) -> String {
        format!("{}.image", self.image_id)
    }

    pub fn thumbneil_file_name(file_name_part: &FileStem) -> FullFileName {
        FullFileName(format!("{}_thumb.jpg", file_name_part.0))
    }

    pub fn get_image_dir_path(conn: &Connection, workspace_id: &str, image_id: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let workspace_dir_path = DBConfig::get_workspace_path(conn, workspace_id)?;
        let workspace_dir_path = Path::new(&workspace_dir_path);
        let images_dir_path = workspace_dir_path.join("images");
        let image_dir_path = images_dir_path.join(format!("{}.image", image_id));
        
        Ok(image_dir_path)
    }

    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(json_file);
        let image_info = serde_json::from_reader(reader)?;
        Ok(image_info)
    }
    
    pub fn save_from_db<T: Writer>(writer: &mut T, workspace: &WorkspaceInfo, image_info: &DBImageInfo) -> Result<(), std::io::Error> {
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
        writer.save(image_info.get_file_image_info_path(workspace), json)?;
        Ok(())
    }
    
    pub fn split_file_name(file_name: &FullFileName) -> Result<(FileStem, String), Box<dyn std::error::Error>> {
        let path = std::path::Path::new(&file_name.0);
        let ext_str = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };
        let ext_str = match ext_str {
            Some(ext) => ext,
            None => {
                return Err(Box::new(ModelError::new("get extension error.".to_string())));
            },
        };

        let file_name_part = file_name.0.trim_end_matches(&format!(".{}", ext_str));

        Ok((FileStem(file_name_part.to_string()), ext_str.to_string()))
    }

    // TODO:共通化できるかも
    fn to_json(&self) -> Result<String, std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thumbnail_file_name() {
        let file_name = FileStem("test.hoge".to_string());
        let thumbneil_file_name = ImageInfo::thumbneil_file_name(&file_name);
        assert_eq!(thumbneil_file_name, FullFileName("test.hoge_thumb.jpg".to_string()));
    }

    #[test]
    fn test_split_file_name() {
        let file_name = FullFileName("test.hoge.png".to_string());
        let (file_name_part, ext) = ImageInfo::split_file_name(&file_name).unwrap();
        assert_eq!(file_name_part, FileStem("test.hoge".to_string()));
        assert_eq!(ext, "png");
    }
}

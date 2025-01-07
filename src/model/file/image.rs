use std::path::Path;
use image::imageops::FilterType;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use image::{DynamicImage, ImageFormat};
use crate::model::db::config::Config as DBConfig;
use crate::model::db::image_info::ImageInfo as DBImageInfo;
use crate::model::file::writer::Writer;

#[derive(Debug)]
struct ImageError {
    pub message: String
}

impl ImageError  {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ImageError  {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WorkspaceError: {}", self.message)
    }
}

impl std::error::Error for ImageError  {}

#[derive(Serialize, Deserialize)]
pub struct Image {
}

impl Image {
    const THUMB_HEIGHT_SIZE: u32 = 300;

    pub fn save<T: Writer>(
        conn: &Connection,
        writer: &mut T,
        workspace_id: &str,
        db_image: &DBImageInfo,
        file_name: &str,
        image_reader: &DynamicImage,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir_path = Self::get_workspace_path(conn, workspace_id)?;
        let workspace_dir_path = Path::new(&workspace_dir_path);
        let images_dir_path = workspace_dir_path.join("images");
        let image_dir_path = images_dir_path.join(format!("{}.image", &db_image.image_id));
        let image_original_file_path = image_dir_path.join(file_name);
        let image_thumbneil_file_path = image_dir_path.join(Self::thumbneil_file_name(file_name)?);
        
        log::info!("save image: {:?}", image_dir_path);

        writer.write_file(image_original_file_path, data)?;
        
        let thumb_with = ((Self::THUMB_HEIGHT_SIZE * image_reader.width()) as f64 / image_reader.height() as f64) as u32;
        let thumb_image = image_reader.resize_exact(thumb_with, Self::THUMB_HEIGHT_SIZE, FilterType::Lanczos3);
        let thumb_image = thumb_image.to_rgb8();
        let mut buffer = std::io::Cursor::new(Vec::new());
        thumb_image.write_to(&mut buffer, ImageFormat::Jpeg)?;
        writer.write_file(image_thumbneil_file_path, &buffer.into_inner())?;

        Ok(())
    }

    pub fn get_workspace_path(conn: &Connection, workspace_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        match DBConfig::find(conn, workspace_id.to_string()) {
            Ok(Some(config)) => Ok(config.path),
            Ok(None) => {
                log::error!("workspace not found.");
                Err(Box::new(ImageError::new("workspace not found.".to_string())))
            },
            Err(err) => {
                log::error!("find workspace error: {}", err);
                Err(Box::new(ImageError::new("find workspace error.".to_string())))
            },
        }
    }
    
    pub fn thumbneil_file_name(file_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = std::path::Path::new(&file_name);
        let ext_str = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };
        let ext_str = match ext_str {
            Some(ext) => ext,
            None => {
                return Err(Box::new(ImageError::new("get extension error.".to_string())));
            },
        };
        let file_name = match path.file_name() {
            Some(file_name) => {
                match file_name.to_str() {
                    Some(file_name) => file_name,
                    None => {
                        return Err(Box::new(ImageError::new("get file name error.".to_string())));
                    }
                }
            },
            None => {
                return Err(Box::new(ImageError::new("get file name error.".to_string())));
            }
        };
        let file_name_part: String = file_name[..file_name.len() - ext_str.len() - 1].to_string();
        let file_name = format!("{}_thumb.jpg", file_name_part);
        Ok(file_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thumbnail_file_name() {
        let file_name = "test.jpg";
        let thumbneil_file_name = Image::thumbneil_file_name(file_name).unwrap();
        assert_eq!(thumbneil_file_name, "test_thumb.jpg");
    }

    #[test]
    fn test_thumbnail_file_name_png() {
        let file_name = "test.png";
        let thumbneil_file_name = Image::thumbneil_file_name(file_name).unwrap();
        assert_eq!(thumbneil_file_name, "test_thumb.jpg");
    }
}

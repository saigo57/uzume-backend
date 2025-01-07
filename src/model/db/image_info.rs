use std::path::Path;
use std::error::Error;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use image::DynamicImage;
use chrono::prelude::*;
use crate::model::file;
use crate::model::file::image::Image as FileImage;
use crate::model::db::config::Config as DBConfig;
use crate::model::entity::image::Image;

#[derive(Debug)]
struct ImageInfoError {
    pub message: String
}

impl ImageInfoError  {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ImageInfoError  {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageInfoError: {}", self.message)
    }
}

impl std::error::Error for ImageInfoError  {}

#[derive(Serialize, Deserialize, Clone)]
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
    pub fn create(conn: &Connection, workspace_id: &str, file_name: &str, image_reader: &DynamicImage) -> Result<Self, Box<dyn Error>> {
        let path = std::path::Path::new(&file_name);
        let ext_str = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };
        let ext_str = match ext_str {
            Some(ext) => ext,
            None => {
                log::error!("get extension error.");
                return Err(Box::new(ImageInfoError::new("get extension error.".to_string())));
            },
        };
        
        let now = Utc::now();
        let jst_time = now.with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).expect("Invalid offset"));
        
        let file_name_without_ext = file_name.trim_end_matches(&format!(".{}", ext_str));

        let image = Self {
            image_id: uuid::Uuid::new_v4().to_string(),
            file_name: file_name_without_ext.to_string(),
            ext: ext_str.to_string(),
            width: image_reader.width(),
            height: image_reader.height(),
            created_at: jst_time.format("%Y-%m-%dT%H:%M:%S%.f%:z").to_string(),
            tags: vec![],
        };
        
        conn.execute("
            INSERT INTO image (
                workspace_id
                ,image_id
                ,file_name
                ,ext
                ,width
                ,height
                ,created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ", params![
            workspace_id,
            image.image_id,
            image.file_name,
            image.ext,
            image.width,
            image.height,
            image.created_at,
        ])?;
        
        Ok(image)
    }

    pub fn get(conn: &Connection, workspace_id: String, page: u32) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut stmt = conn.prepare("
            SELECT
                image_id
                ,file_name
                ,ext
                ,width
                ,height
                ,created_at 
            FROM image
            WHERE workspace_id = ?1 
            LIMIT 100
            OFFSET ?2
        ")?;
        let offset = 100 * (page - 1);
        let images = stmt.query_map(params![workspace_id, offset], |row| {
            Ok(Self {
                image_id: row.get(0)?,
                file_name: row.get(1)?,
                ext: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                created_at: row.get(5)?,
                tags: vec![],
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    pub fn find(conn: &Connection, workspace_id: &String, image_id: &String) -> Result<Option<Self>, Box<dyn Error>> {
        let mut stmt = conn.prepare("
            SELECT
                image_id
                ,file_name
                ,ext
                ,width
                ,height
                ,created_at 
            FROM image
            WHERE workspace_id = ?1 AND image_id = ?2
        ")?;
        let image = stmt.query_map(params![workspace_id, image_id], |row| {
            Ok(Self {
                image_id: row.get(0)?,
                file_name: row.get(1)?,
                ext: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                created_at: row.get(5)?,
                tags: vec![],
            })
        })?.next();
        
        match image {
            Some(Ok(image)) => Ok(Some(image)),
            Some(Err(e)) => Err(Box::new(e)),
            None => Ok(None),
        }
    }

    pub fn get_image(&self, conn: &Connection, workspace_id: &str) -> Result<Image, Box<dyn std::error::Error>> {
        let image_dir_path = self.get_image_dir_path(&conn, &workspace_id)?;
        let image_original_file_path = image_dir_path.join(format!("{}.{}", self.file_name, self.ext));
        //let image_thumbneil_file_path = image_dir_path.join(FileImage::thumbneil_file_name(&self.file_name)?);
        log::info!("image_original_file_path: {:?}", image_original_file_path);

        let data = std::fs::read(&image_original_file_path)?;
        let ext = match image_original_file_path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };
        let ext = match ext {
            Some(ext) => ext,
            None => {
                log::error!("get extension error.");
                return Err(Box::new(ImageInfoError::new("get extension error".to_string())));
            },
        };
        Ok(Image { data, ext: ext.to_string() })
    }
    
    pub fn get_image_dir_path(&self, conn: &Connection, workspace_id: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let workspace_dir_path = Self::get_workspace_path(conn, workspace_id)?;
        let workspace_dir_path = Path::new(&workspace_dir_path);
        let images_dir_path = workspace_dir_path.join("images");
        let image_dir_path = images_dir_path.join(format!("{}.image", self.image_id));
        
        Ok(image_dir_path)
    }

    pub fn get_workspace_path(conn: &Connection, workspace_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        match DBConfig::find(conn, workspace_id.to_string()) {
            Ok(Some(config)) => Ok(config.path),
            Ok(None) => {
                log::error!("workspace not found.");
                Err(Box::new(ImageInfoError::new("workspace not found.".to_string())))
            },
            Err(err) => {
                log::error!("find workspace error: {}", err);
                Err(Box::new(ImageInfoError::new("find workspace error.".to_string())))
            },
        }
    }
}

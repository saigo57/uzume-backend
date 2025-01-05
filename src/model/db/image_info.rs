use std::error::Error;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use image::DynamicImage;
use chrono::prelude::*;

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

        let image = Self {
            image_id: uuid::Uuid::new_v4().to_string(),
            file_name: file_name.to_string(),
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
}

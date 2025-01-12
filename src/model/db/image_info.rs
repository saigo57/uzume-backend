use std::error::Error;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use image::ImageReader;
use std::io::Cursor;
use crate::model::db::config::Config as DBConfig;
use crate::model::db::tag::Tag as DBTag;
use crate::model::db::image_tag_map::ImageTagMap as DBImageTagMap;
use crate::model::file::image::Image as FileImage;
use crate::model::file::image_info::{ImageInfo as FileImageInfo, FullFileName, FileStem};
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::entity::image::Image;
use crate::model::file::writer::Writer;
use crate::util::{jst_time_string, ModelError};

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub image_id: String,

    pub file_name: FileStem,

    pub ext: String,

    pub width: u32,

    pub height: u32,

    pub created_at: String,
}

impl ImageInfo {
    pub fn insert(&self, conn: &Connection, workspace_id: &str) -> Result<(), Box<dyn Error>> {
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
            self.image_id,
            self.file_name.0,
            self.ext,
            self.width,
            self.height,
            self.created_at,
        ])?;

        Ok(())
    }

    pub fn create<T: Writer>(
        conn: &Connection,
        writer: &mut T,
        workspace_info: &WorkspaceInfo,
        file_name: &FullFileName,
        image_data: &[u8],
    ) -> Result<Self, Box<dyn Error>> {
        let (file_name_part, ext) = FileImageInfo::split_file_name(file_name)?;

        let cursor = Cursor::new(image_data);
        let image_reader = ImageReader::new(cursor)
            .with_guessed_format()?
            .decode()?;
        
        let image = Self {
            image_id: uuid::Uuid::new_v4().to_string(),
            file_name: file_name_part,
            ext,
            width: image_reader.width(),
            height: image_reader.height(),
            created_at: jst_time_string()?,
        };
        
        image.insert(conn, &workspace_info.workspace_id)?;

        FileImageInfo::save_from_db(
            conn,
            &mut writer.clone(),
            workspace_info,
            &image
        )?;
        FileImage::save(
            conn,
            &mut writer.clone(),
            &workspace_info.workspace_id,
            &image,
            &image_reader,
            image_data,
        )?;
        
        Ok(image)
    }
    
    pub fn add_tag<T: Writer>(&self, conn: &Connection, writer: &mut T, workspace_id: &str, tag_id: &str) -> Result<(), Box<dyn Error>> {
        let tag = match DBTag::find(conn, workspace_id.to_string(), tag_id.to_string()) {
            Ok(tag) => {
                match tag {
                    Some(tag) => tag,
                    None => {
                        log::error!("tag not found. workspace_id: {}, tag_id: {}", workspace_id, tag_id);
                        return Err(Box::new(ModelError::new("tag not found".to_string())));
                    }
                }
            },
            Err(e) => {
                log::error!("tag not found. workspace_id: {}, tag_id: {}, e: {}", workspace_id, tag_id, e);
                return Err(Box::new(ModelError::new("tag not found".to_string())));
            }
        };
        
        let workspace = match DBConfig::find(conn, workspace_id.to_string())? {
            Some(workspace) => workspace,
            None => {
                log::error!("workspace not found. workspace_id: {}", workspace_id);
                return Err(Box::new(ModelError::new("workspace not found".to_string())));
            }
        };
        
        DBImageTagMap::add(conn, &self.image_id, &tag.tag_id)?;
        
        FileImageInfo::save_from_db(
            conn,
            &mut writer.clone(),
            &workspace,
            self
        )?;

        Ok(())
    }
    
    pub fn remove_tag<T: Writer>(&self, conn: &Connection, writer: &mut T, workspace_id: &str, tag_id: &str) -> Result<(), Box<dyn Error>> {
        let workspace = match DBConfig::find(conn, workspace_id.to_string())? {
            Some(workspace) => workspace,
            None => {
                log::error!("workspace not found. workspace_id: {}", workspace_id);
                return Err(Box::new(ModelError::new("workspace not found".to_string())));
            }
        };
        
        DBImageTagMap::remove(conn, &self.image_id, tag_id)?;
        
        FileImageInfo::save_from_db(
            conn,
            &mut writer.clone(),
            &workspace,
            self
        )?;

        Ok(())
    }
    
    pub fn tag_ids(&self, conn: &Connection) -> Result<Vec<String>, Box<dyn Error>> {
        DBImageTagMap::get_tag_id(conn, &self.image_id)
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
                file_name: FileStem(row.get(1)?),
                ext: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                created_at: row.get(5)?,
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
                file_name: FileStem(row.get(1)?),
                ext: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?.next();
        
        match image {
            Some(Ok(image)) => Ok(Some(image)),
            Some(Err(e)) => Err(Box::new(e)),
            None => Ok(None),
        }
    }

    pub fn get_image(&self, conn: &Connection, workspace_id: &str, thumbnail: bool) -> Result<Image, Box<dyn std::error::Error>> {
        let image_dir_path = FileImageInfo::get_image_dir_path(conn, workspace_id, &self.image_id)?;
        let image_file_path =
            if thumbnail {
                image_dir_path.join(self.get_thumbnail_file_name().0)
            } else {
                image_dir_path.join(format!("{}.{}", self.file_name.0, self.ext))
            };

        let data = std::fs::read(&image_file_path)?;
        let ext =
            if thumbnail {
                "jpg"
            } else {
                &self.ext
            };
        Ok(Image { data, ext: ext.to_string() })
    }
    
    pub fn get_file_name(&self) -> FullFileName {
        FullFileName(format!("{}.{}", self.file_name.0, self.ext))
    }

    pub fn get_thumbnail_file_name(&self) -> FullFileName {
        FileImageInfo::thumbneil_file_name(&self.file_name)
    }
}

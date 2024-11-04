use std::error::Error;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};

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

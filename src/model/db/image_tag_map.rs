use std::error::Error;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageTagMap {
    pub image_id: String,
    pub tag_id: String,
}

impl ImageTagMap {
    pub fn add(conn: &Connection, image_id: &str, tag_id: &str) -> Result<(), Box<dyn Error>> {
        let tag_id_list = Self::get_tag_id(conn, image_id)?;
        if tag_id_list.contains(&tag_id.to_string()) {
            // 重複してたら何もしない
            return Ok(());
        }

        conn.execute("
            INSERT INTO image_tag_map (image_id, tag_id) VALUES (?1, ?2)
        ", params![
            image_id,
            tag_id,
        ])?;

        Ok(())
    }

    pub fn remove(conn: &Connection, image_id: &str, tag_id: &str) -> Result<(), Box<dyn Error>> {
        conn.execute("
            DELETE FROM image_tag_map WHERE image_id = ?1 AND tag_id = ?2
        ", params![
            image_id,
            tag_id,
        ])?;

        Ok(())
    }
    
    pub fn get_tag_id(conn: &Connection, image_id: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut stmt = conn.prepare("
            SELECT tag_id FROM image_tag_map WHERE image_id = ?1
        ")?;
        let tag_id_iter = stmt.query_map(params![image_id], |row| {
            let item = row.get(0)?;
            Ok(item)
        })?;

        let mut tag_id_list = Vec::new();
        for tag_id in tag_id_iter {
            tag_id_list.push(tag_id?);
        }

        Ok(tag_id_list)
    }
}

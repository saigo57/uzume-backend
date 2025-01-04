use std::error::Error;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub workspace_id: String,

    pub tag_id: String,

    pub name: String,

    pub favorite: bool,

    pub tag_group_id: String,
}

impl Tag {
    const TAG_SELECT: &str = "
        SELECT
            workspace_id
            ,tag_id
            ,name
            ,favorite
            ,tag_group_id
    ";

    pub fn get_all(conn: &Connection, workspace_id: String) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut stmt = conn.prepare(&format!("
            {}
            FROM tag
            WHERE workspace_id = ?1
        ", Self::TAG_SELECT))?;
        let tags: Vec::<Tag> = stmt.query_map(params![workspace_id], |row| {
            let favorite: i32 = row.get(3)?;
            Ok(Tag {
                workspace_id: row.get(0)?,
                tag_id: row.get(1)?,
                name: row.get(2)?,
                favorite: favorite != 0,
                tag_group_id: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }
    
    pub fn find(conn: &Connection, workspace_id: String, tag_id: String) -> Result<Option<Self>, Box<dyn Error>> {
        let mut stmt = conn.prepare(&format!("
            {}
            FROM tag
            WHERE workspace_id = ?1 AND tag_id = ?2
        ", Self::TAG_SELECT))?;
        let tag = stmt.query_map(params![workspace_id, tag_id], |row| {
            let favorite: i32 = row.get(3)?;
            Ok(Tag {
                workspace_id: row.get(0)?,
                tag_id: row.get(1)?,
                name: row.get(2)?,
                favorite: favorite != 0,
                tag_group_id: row.get(4)?,
            })
        })?.next();
        match tag {
            Some(Ok(tag)) => Ok(Some(tag)),
            Some(Err(e)) => Err(Box::new(e)),
            None => Ok(None),
        }
    }

    pub fn create(conn: &Connection, workspace_id: String, name: String) -> Result<Self, Box<dyn Error>> {
        let tag = Self {
            workspace_id,
            tag_id: uuid::Uuid::new_v4().to_string(),
            name,
            favorite: false,
            tag_group_id: "".to_string(),
        };

        conn.execute("
            INSERT INTO tag (
                workspace_id
                ,tag_id
                ,name
                ,favorite
                ,tag_group_id
            ) VALUES (?1, ?2, ?3, ?4, ?5)
        ", params![
            tag.workspace_id,
            tag.tag_id,
            tag.name,
            tag.favorite_to_i(),
            tag.tag_group_id,
        ])?;

        Ok(tag)
    }
    
    pub fn favorite_to_i(&self) -> i32 {
        if self.favorite { 1 } else { 0 }
    }
    
    pub fn save(&self, conn: &Connection) -> Result<(), Box<dyn Error>> {
        conn.execute("
            UPDATE tag
            SET
                name = ?3
                ,favorite = ?4
                ,tag_group_id = ?5
            WHERE workspace_id = ?1 AND tag_id = ?2
        ", params![
            self.workspace_id,
            self.tag_id,
            self.name,
            self.favorite_to_i(),
            self.tag_group_id,
        ])?;
        Ok(())
    }
}

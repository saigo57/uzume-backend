use std::error::Error;
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Tag {
    #[schema(example = "d0bf74e3-5ab3-4b2c-8479-5d5069d4aea9")]
    pub workspace_id: String,

    #[schema(example = "d0bf74e3-5ab3-4b2c-8479-5d5069d4aea9")]
    pub tag_id: String,

    #[schema(example = "タグ名")]
    pub name: String,

    #[schema(example = "true")]
    pub favorite: bool,

    #[schema(example = "dfbdd496-6b59-44d1-a0e3-b86b454b02bd")]
    pub tag_group_id: String,
}

impl Tag {
    pub fn get_all(conn: &Connection, workspace_id: String) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut stmt = conn.prepare("
            SELECT
                workspace_id
                ,tag_id
                ,name
                ,favorite
                ,tag_group_id
            FROM tag
            WHERE workspace_id = ?1
        ")?;
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

    pub fn create(conn: &Connection, workspace_id: String, name: String) -> Result<Self, Box<dyn Error>> {
        let tag = Self {
            workspace_id,
            tag_id: uuid::Uuid::new_v4().to_string(),
            name,
            favorite: false,
            tag_group_id: "".to_string(),
        };
        tag.save(conn)?;
        Ok(tag)
    }

    pub fn save(&self, conn: &Connection) -> Result<(), Box<dyn Error>> {
        conn.execute("
            INSERT INTO tag (
                workspace_id
                ,tag_id
                ,name
                ,favorite
                ,tag_group_id
            ) VALUES (?1, ?2, ?3, ?4, ?5)
        ", params![
            self.workspace_id,
            self.tag_id,
            self.name,
            if self.favorite { 1 } else { 0 },
            self.tag_group_id,
        ])?;
        Ok(())
    }
}

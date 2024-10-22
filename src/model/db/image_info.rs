use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImageInfo {
    #[schema(example = "e3e2ffc1-bee4-401d-a71a-f42faa150c04")]
    pub image_id: String,

    #[schema(example = "file_name")]
    pub file_name: String,

    #[schema(example = "jpg")]
    pub ext: String,

    #[schema(example = 1920)]
    pub width: u32,

    #[schema(example = 1080)]
    pub height: u32,

    #[schema(example = "2024-07-30T21:56:33.2140303+09:00")]
    pub created_at: String,

    #[schema(example = "\"a0b257bb-b7c6-46f3-b27c-31f8ce1c3703\", \"29fccd85-524b-4337-a780-f44fae90bc19\"")]
    pub tags: Vec<String>,
}

impl ImageInfo {
    pub fn get(conn: &Connection, workspace_id: String, page: u32) -> Result<Vec<Self>, std::io::Error> {
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
        ").unwrap();
        let offset = 100 * (page - 1);
        let images: Vec::<Self> = stmt.query_map(params![workspace_id, offset], |row| {
            Ok(Self {
                image_id: row.get(0)?,
                file_name: row.get(1)?,
                ext: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                created_at: row.get(5)?,
                tags: vec![],
            })
        }).unwrap().map(|r| r.unwrap()).collect();

        Ok(images)
    }
}

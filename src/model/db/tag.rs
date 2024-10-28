use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Tag {
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
    pub fn get(conn: &Connection, workspace_id: String) -> Result<Vec<Self>, std::io::Error> {
        let mut stmt = conn.prepare("
            SELECT
                tag_id
                ,name
                ,favorite
                ,tag_group_id
            FROM tag
            WHERE workspace_id = ?1
        ").unwrap();
        let tags: Vec::<Tag> = stmt.query_map(params![workspace_id], |row| {
            let favorite: i32 = row.get(2)?;
            Ok(Tag {
                tag_id: row.get(0)?,
                name: row.get(1)?,
                favorite: favorite != 0,
                tag_group_id: row.get(3)?,
            })
        }).unwrap().map(|r| r.unwrap()).collect();
        Ok(tags)
    }
}

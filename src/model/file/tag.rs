use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub tag_id: String,

    pub name: String,

    pub favorite: bool,

    pub tag_group_id: String,
}

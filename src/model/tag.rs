use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

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

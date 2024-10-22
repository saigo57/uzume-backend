use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

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
    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(file_path).unwrap();
        let reader = std::io::BufReader::new(json_file);
        let image_info = serde_json::from_reader(reader).unwrap();
        Ok(image_info)
    }
}

use serde::{Serialize, Deserialize};

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
    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(json_file);
        let image_info = serde_json::from_reader(reader)?;
        Ok(image_info)
    }
}

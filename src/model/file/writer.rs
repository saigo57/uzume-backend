use std::io::Write;
use std::sync::{Arc, Mutex};

pub trait Writer: Send + Sync + Clone {
    fn save(&self, path: std::path::PathBuf, json: String) -> Result<(), std::io::Error>;
}

#[derive(Clone)]
pub struct FileWriter;

impl Writer for FileWriter {
    fn save(&self, path: std::path::PathBuf, json: String) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[allow(dead_code)] // テスト用コード
#[derive(Debug)]
pub struct MockWriteData {
    pub path: String,
    pub json: String,
}

#[derive(Clone)]
pub struct MockWriter {
    pub data: Arc<Mutex<Option<MockWriteData>>>,
}

#[allow(dead_code)] // テスト用コード
impl MockWriter {
    pub fn get_path(&self) -> String {
        let data = self.data.lock().unwrap();
        let data = data.as_ref().unwrap();
        data.path.clone()
    }

    pub fn get_json(&self) -> String {
        let data = self.data.lock().unwrap();
        let data = data.as_ref().unwrap();
        data.json.clone()
    }
}

impl Writer for MockWriter {
    fn save(&self, path: std::path::PathBuf, json: String) -> Result<(), std::io::Error> {
        *self.data.lock().unwrap() = Some(MockWriteData { path: path.to_string_lossy().to_string(), json });
        Ok(())
    }
}

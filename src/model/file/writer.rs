use std::io::Write;
use std::sync::{Arc, Mutex};

pub trait Writer: Send + Sync + Clone {
    fn save(&self, path: std::path::PathBuf, json: String) -> Result<(), std::io::Error>;
    fn write_file(&self, path: std::path::PathBuf, data: &[u8]) -> Result<(), std::io::Error>;
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
    
    fn write_file(&self, path: std::path::PathBuf, data: &[u8]) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(path)?;
        file.write_all(data)?;
        Ok(())
    }
}

#[allow(dead_code)] // テスト用コード
#[derive(Debug)]
pub struct MockWriteData {
    pub path: String,
    pub data: String,
}

#[derive(Clone)]
pub struct MockWriter {
    pub history: Arc<Mutex<Vec<MockWriteData>>>,
}

#[allow(dead_code)] // テスト用コード
impl MockWriter {
}

impl Writer for MockWriter {
    fn save(&self, path: std::path::PathBuf, json: String) -> Result<(), std::io::Error> {
        let mut history = self.history.lock().unwrap();
        history.push(MockWriteData {
            path: path.to_string_lossy().to_string(),
            data: json,
        });
        Ok(())
    }
    
    fn write_file(&self, path: std::path::PathBuf, data: &[u8]) -> Result<(), std::io::Error> {
        let mut history = self.history.lock().unwrap();
        history.push(MockWriteData {
            path: path.to_string_lossy().to_string(),
            data: String::from_utf8_lossy(data).to_string()
        });

        Ok(())
    }
}

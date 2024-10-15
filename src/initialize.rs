
use rusqlite::{ Connection, Error };
use std::path::Path;
use std::sync::Arc;
use std::fs;
use tokio::sync::Mutex;
use crate::model::config::Config;
use crate::model::image_info::ImageInfo;

pub async fn initialize(conn: Arc<Mutex<Connection>>) -> Result<(), Error> {
    let conn = conn.lock().await;
    let config = Config::new().unwrap();
    load_image_info(&conn, &config).await?;
    Ok(())
}

async fn load_image_info(conn: &Connection, config: &Config) -> Result<(), Error> {
    for workspace in &config.workspace_list {
        let workspace_path = workspace.clone().path.clone();
        let workspace_path = Path::new(&workspace_path);
        let images_path = workspace_path.join("images");
        let entries = fs::read_dir(images_path.clone()).unwrap();

        for entry in entries {
            let entry = entry.unwrap();
            let image_dir_name = entry.file_name();
            let image_info_file_path = images_path.join(image_dir_name).join("imageinfo.json");
            let image_info = ImageInfo::load(image_info_file_path.to_str().unwrap()).unwrap();

            conn.execute(
                "INSERT INTO image (workspace_id, image_id, file_name, ext, width, height, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                [
                    workspace.workspace_id.clone(),
                    image_info.image_id,
                    image_info.file_name,
                    image_info.ext,
                    image_info.width.to_string(),
                    image_info.height.to_string(),
                    image_info.created_at,
                ],
            ).unwrap();
        }
    }

    Ok(())
}

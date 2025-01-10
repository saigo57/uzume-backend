use image::imageops::FilterType;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use image::{DynamicImage, ImageFormat};
use crate::model::db::image_info::ImageInfo as DBImageInfo;
use crate::model::file::image_info::ImageInfo as FileImageInfo;
use crate::model::file::writer::Writer;

#[derive(Serialize, Deserialize)]
pub struct Image {
}

impl Image {
    const THUMB_HEIGHT_SIZE: u32 = 300;

    pub fn save<T: Writer>(
        conn: &Connection,
        writer: &mut T,
        workspace_id: &str,
        db_image: &DBImageInfo,
        image_reader: &DynamicImage,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let image_dir_path = FileImageInfo::get_image_dir_path(conn, workspace_id, &db_image.image_id)?;
        let image_original_file_path = image_dir_path.join(&db_image.get_file_name().0);
        let image_thumbneil_file_path = image_dir_path.join(&db_image.get_thumbnail_file_name().0);
        
        writer.write_file(image_original_file_path, data)?;
        
        let thumb_with = ((Self::THUMB_HEIGHT_SIZE * image_reader.width()) as f64 / image_reader.height() as f64) as u32;
        let thumb_image = image_reader.resize_exact(thumb_with, Self::THUMB_HEIGHT_SIZE, FilterType::Lanczos3);
        let thumb_image = thumb_image.to_rgb8();
        let mut buffer = std::io::Cursor::new(Vec::new());
        thumb_image.write_to(&mut buffer, ImageFormat::Jpeg)?;
        writer.write_file(image_thumbneil_file_path, &buffer.into_inner())?;

        Ok(())
    }
}

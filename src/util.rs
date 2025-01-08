use chrono::prelude::*;
use serde::Serialize;
use axum::{
    http,
    http::StatusCode,
    response::Response,
    body::Body,
    Json,
};
use crate::model::entity::image::Image;

pub type ApiResponse<T> = Result<Json<T>, Json<BasicApiError>>;

#[derive(Debug, Serialize)]
pub struct BasicApiError {
    pub error_message: String,
}

#[derive(Debug)]
pub struct ModelError {
    pub message: String
}

impl ModelError  {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ModelError  {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ModelError: {}", self.message)
    }
}

impl std::error::Error for ModelError  {}

pub fn build_image_response(image: &Image) -> Result<Response, http::Error> {
    let res = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", format!("image/{}", image.ext))
        .body(Body::from(image.data.clone()))?;

    Ok(res)
}

pub fn jst_time_string() -> Result<String, String> {
    let now = Utc::now();
    let offset = match chrono::FixedOffset::east_opt(9 * 3600) {
        Some(offset) => offset,
        None => return Err("Invalid offset".to_string()),
    };
    let jst_time = now.with_timezone(&offset);
    
    Ok(jst_time.format("%Y-%m-%dT%H:%M:%S%.f%:z").to_string())
}

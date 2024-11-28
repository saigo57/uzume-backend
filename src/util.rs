use serde::Serialize;
use axum::Json;

pub type ApiResponse<T> = Result<Json<T>, Json<BasicApiError>>;

#[derive(Debug, Serialize)]
pub struct BasicApiError {
    pub error_message: String,
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BasicApiError {
    pub error_message: String,
}

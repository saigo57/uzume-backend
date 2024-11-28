use axum::extract::Multipart;
use bytes::Bytes;


#[derive(Debug)]
struct MultipartParamsError {
    pub message: String
}

impl MultipartParamsError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for MultipartParamsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MultipartParamsError: {}", self.message)
    }
}

impl std::error::Error for MultipartParamsError {}

#[allow(dead_code)] // 利用先が未実装
pub struct MultipartText {
    pub param_name: String,
    pub data: String,
}

pub struct MultipartFile {
    pub param_name: String,
    pub file_name: String,
    pub data: Bytes,
}

pub struct MultipartParams {
    pub files: Vec<MultipartFile>,
    pub texts: Vec<MultipartText>,
}

impl MultipartParams {
    pub async fn new(multipart: &mut Multipart) -> Result<Self, Box<dyn std::error::Error>> {
        let mut params = Self { files: vec![], texts: vec![] };
        
        loop {
            let field = multipart.next_field().await?;
            let field = match field {
                Some(field) => field,
                None => break,
            };
            
            let param_name = match field.name() {
                Some(name) => name.to_string(),
                None => {
                    return Err(Box::new(MultipartParamsError::new("param_name is None".to_string())));
                },
            };
            
            match field.content_type() {
                Some(_) => {
                    let file_name = match field.file_name() {
                        Some(file_name) => {
                            file_name.to_owned()
                        },
                        None => {
                            return Err(Box::new(MultipartParamsError::new("file_name is None".to_string())));
                        },
                    };
                    let data = field.bytes().await?;
                    params.files.push(MultipartFile { param_name, file_name, data });
                },
                None => {
                    let data = field.text().await?;
                    params.texts.push(MultipartText { param_name, data });
                },
            }
        }
        
        Ok(params)
    }
}

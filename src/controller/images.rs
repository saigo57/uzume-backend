use std::cmp::max;
use axum::{
    self,
    routing::{get, post},
    http::StatusCode,
    extract::{Extension, Multipart, Path},
    response::{Response, IntoResponse},
    extract::Query,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use image::{ImageReader, DynamicImage};
use std::io::Cursor;
use crate::controller::middleware::auth;
use crate::model::db::config::Config as DBConfig;
use crate::model::file::image_info::FullFileName;
use crate::model::db::image_info::ImageInfo as DBImageInfo;
use crate::model::file::writer::Writer;
use crate::util::{build_image_response, ApiResponse, BasicApiError};
use crate::multipart_params::MultipartParams;

#[derive(Serialize, Deserialize)]
struct GetImageListParams {
    page: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ImageSize {
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "thumbnail")]
    Thumbnail,
}

#[derive(Serialize, Deserialize)]
struct GetImageParams {
    image_size: Option<ImageSize>,
}

#[derive(Serialize, Deserialize)]
struct ImagesResponse {
    page: u32,
    images: Vec<DBImageInfo>,
}

async fn get_images(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Query(query): Query<GetImageListParams>,
) -> (StatusCode, ApiResponse<ImagesResponse>) {
    let conn = conn.lock().await;
    // queryを解釈してpage変数を新しく作る。どこかで値がなかった場合は1を入れる
    let page = query.page.unwrap_or(1);
    let page = max(1, page); // pageは1以上
    let images = match DBImageInfo::get(&conn, workspace_id.clone(), page) {
        Ok(images) => images,
        Err(e) => {
            log::error!("{}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: e.to_string() }))
            );
        }
    };
    let ir = ImagesResponse {
        page,
        images,
    };
    (StatusCode::OK, Ok(Json(ir)))
}

fn decode_image_reader(binary_data: &[u8]) -> Result<DynamicImage, image::ImageError> {
    let cursor = Cursor::new(binary_data);
    let image = ImageReader::new(cursor)
        .with_guessed_format()?
        .decode()?;

    Ok(image)
}

async fn post_images<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    mut multipart: Multipart,
) -> (StatusCode, ApiResponse<()>) {
    let conn = conn.lock().await;

    let multipart_params = match MultipartParams::new(&mut multipart).await {
        Ok(params) => params,
        Err(err) => {
            log::error!("MultipartParams error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: "MultipartParams error.".to_string() }))
            );
        },
    };
    
    if multipart_params.files.len() != 1 {
        log::error!("Invalid file count: {}", multipart_params.files.len());
        return (
            StatusCode::BAD_REQUEST,
            Err(Json(BasicApiError { error_message: "Invalid file count".to_string() }))
        );
    }
    
    let image_field = &multipart_params.files[0];
    if image_field.param_name != "image" {
        log::error!("Invalid parameter name: {}", image_field.param_name);
        return (
            StatusCode::BAD_REQUEST,
            Err(Json(BasicApiError { error_message: "Invalid parameter name".to_string() }))
        );
    }

    let image_reader = match decode_image_reader(&image_field.data) {
        Ok(reader) => reader,
        Err(err) => {
            log::error!("decode image error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: "decode image error.".to_string() }))
            );
        },
    };

    let workspace = match DBConfig::find(&conn, workspace_id.clone()) {
        Ok(workspace) => workspace,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    let workspace = match workspace {
        Some(workspace) => workspace,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: format!("workspace({workspace_id}) not found.") }))
            );
        }
    };

    let db_image = match DBImageInfo::create(
        &conn,
        &mut writer.clone(),
        &workspace,
        &FullFileName(image_field.file_name.clone()),
        &image_field.data,
    ) {
        Ok(db_image) => { db_image },
        Err(err) => {
            log::error!("save icon error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: "save icon error.".to_string() }))
            );
        },
    };
    
    (StatusCode::CREATED, Ok(Json(())))
}

async fn get_image(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Path(id): Path<String>,
    Query(query): Query<GetImageParams>,
) -> Response {
    let conn = conn.lock().await;

    let image_size = query.image_size.unwrap_or(ImageSize::Original);
    
    let image = match DBImageInfo::find(&conn, &workspace_id, &id) {
        Ok(image) => image,
        Err(err) => {
            log::error!("find image error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BasicApiError { error_message: "find image error.".to_string() })
            )
            .into_response();
        },
    };
    
    let image = match image {
        Some(image) => image,
        None => {
            log::info!("image not found: {}", id);
            return (
                StatusCode::NOT_FOUND,
                Json(BasicApiError { error_message: "image not found.".to_string() })
            )
            .into_response();
        },
    };
    
    let image = match image.get_image(&conn, &workspace_id, image_size == ImageSize::Thumbnail) {
        Ok(image) => image,
        Err(err) => {
            log::info!("get image: {}", err);
            return (
                StatusCode::NOT_FOUND,
                ()
            )
            .into_response();
        },
    };

    match build_image_response(&image) {
        Ok(res) => res,
        Err(err) => {
            log::info!("get image: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BasicApiError { error_message: "get image error.".to_string() })
            )
            .into_response();
        },
    }
}

// GET    /images?page=1
// *POST   /images
// GET    /images/{id}/file?image_size=original
// (PATCH /images/{id}
// PATCH  /images/{image_id}/tags/{tag_id}
// DELETE /images/{image_id}/tags/{tag_id}
pub fn router<T: Writer + 'static>(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/", get(get_images))
        .route("/", post(post_images::<T>))
        .route("/:id/file", get(get_image))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use crate::test_util::TestUtil;
    use crate::model::file::image_info::{ImageInfo as FileImageInfo, FileStem};

    mod test_post_images {
        use super::*;
        use hyper;
        use axum::body::Body;
        use axum::extract::Request;
        use axum::extract::FromRequest;
        use chrono::DateTime;

        fn is_iso8601_format(input: &str) -> bool {
            DateTime::parse_from_rfc3339(input).is_ok()
        }

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;

            {
                let conn = tu.conn.lock().await;
                let images = DBImageInfo::get(&conn, tu.workspace_id.clone(), 1).unwrap();

                assert_eq!(images.len(), 0);
            }
            
            let image_data = include_bytes!("../../test_data/computer_server1.png");
            let boundary = "testboundary";
            let body = [
                format!(
                    "--{}\r\n\
                     Content-Disposition: form-data; name=\"image\"; filename=\"computer_server1.png\"\r\n\
                     Content-Type: image/png\r\n\r\n",
                    boundary,
                ).as_bytes(),
                image_data,
                "\r\n".as_bytes(),
                format!("--{}--\r\n", boundary).as_bytes()
            ].concat();
            let request = Request::builder()
                .header(
                    hyper::header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap();
            let multipart = Multipart::from_request(request, &()).await.unwrap();

            let (status, _result) = post_images(
                Extension(tu.workspace_id.clone()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                multipart,
            ).await;
            
            assert_eq!(status, StatusCode::CREATED);
            
            {
                let history = tu.writer.history.lock().unwrap();
                assert_eq!(history.len(), 3);

                assert!(history[0].path.ends_with("/imageinfo.json"));
                let file_result: FileImageInfo = serde_json::from_str(&history[0].data).unwrap();
                assert_eq!(file_result.image_id.len(), 36);
                assert_eq!(file_result.file_name, FileStem("computer_server1".to_string()));
                assert_eq!(file_result.ext, "png");
                assert_eq!(file_result.width, 338);
                assert_eq!(file_result.height, 400);
                assert!(is_iso8601_format(file_result.created_at.as_str()));
                assert!(file_result.tags.is_empty());

                let re = Regex::new(r"^/path/to/test\.uzume/images/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\.image/computer_server1\.png$").unwrap();
                assert!(re.is_match(&history[1].path));
                assert!(!history[1].data.is_empty());

                let re = Regex::new(r"^/path/to/test\.uzume/images/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\.image/computer_server1_thumb\.jpg$").unwrap();
                assert!(re.is_match(&history[2].path));
                assert!(!history[2].data.is_empty());
                
                assert!(history[1].data.len() > history[2].data.len());
            }
            
            {
                let conn = tu.conn.lock().await;
                let images = DBImageInfo::get(&conn, tu.workspace_id.clone(), 1).unwrap();
                assert_eq!(images.len(), 1);
            }
        }
    }
}

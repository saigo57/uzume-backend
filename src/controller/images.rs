use std::cmp::max;
use axum::{
    self,
    routing::get,
    http::StatusCode,
    extract::Extension,
    extract::Query,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::image_info::ImageInfo;

#[derive(Serialize, Deserialize, ToSchema)]
struct GetImageParams {
    page: Option<u32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct ImagesResponse {
    page: u32,
    images: Vec<ImageInfo>,
}

#[utoipa::path(
    get,
    path = "/api/v1/images",
    responses(
        (status = 200, description = "All images", body = ImagesResponse)
    )
)]
async fn get_images(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Query(query): Query<GetImageParams>,
) -> (StatusCode, Json<ImagesResponse>) {
    let conn = conn.lock().await;
    // queryを解釈してpage変数を新しく作る。どこかで値がなかった場合は1を入れる
    let page = query.page.unwrap_or(1);
    let page = max(1, page); // pageは1以上
    let mut stmt = conn.prepare("SELECT workspace_id, image_id, file_name, ext, width, height, created_at FROM image WHERE workspace_id = ?1 LIMIT 100 OFFSET ?2").unwrap();
    let images: Vec::<ImageInfo> = stmt.query_map(params![workspace_id, 100 * (page - 1)], |row| {
        Ok(ImageInfo {
            image_id: row.get(1)?,
            file_name: row.get(2)?,
            ext: row.get(3)?,
            width: row.get(4)?,
            height: row.get(5)?,
            created_at: row.get(6)?,
            tags: vec![],
        })
    }).unwrap().map(|r| r.unwrap()).collect();
    let ir = ImagesResponse {
        page,
        images,
    };
    (StatusCode::OK, Json(ir))
}

pub fn router(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/", get(get_images))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth))
}

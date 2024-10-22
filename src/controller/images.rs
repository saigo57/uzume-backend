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
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::db::image_info::ImageInfo as DBImageInfo;

#[derive(Serialize, Deserialize, ToSchema)]
struct GetImageParams {
    page: Option<u32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct ImagesResponse {
    page: u32,
    images: Vec<DBImageInfo>,
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
    let images = DBImageInfo::get(&conn, workspace_id.clone(), page).unwrap();
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

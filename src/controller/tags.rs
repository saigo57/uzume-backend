
use axum::{
    self,
    routing::get,
    http::StatusCode,
    extract::Extension,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::db::tag::Tag as DBTag;

#[derive(Serialize, Deserialize, ToSchema)]
struct TagsResponse {
    tags: Vec<DBTag>,
}

#[utoipa::path(
    get,
    path = "/api/v1/tags",
    responses(
        (status = 200, description = "All tags", body = TagsResponse)
    )
)]
async fn get_tags(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> (StatusCode, Json<TagsResponse>) {
    let conn = conn.lock().await;
    let tags = DBTag::get(&conn, workspace_id.clone()).unwrap();
    let tr = TagsResponse {
        tags,
    };
    (StatusCode::OK, Json(tr))
}

pub fn router(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/", get(get_tags))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth))
}


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
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::tag::Tag;

#[derive(Serialize, Deserialize, ToSchema)]
struct TagsResponse {
    tags: Vec<Tag>,
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
    let mut stmt = conn.prepare("SELECT tag_id, name, favorite, tag_group_id FROM tag WHERE workspace_id = ?1").unwrap();
    let tags: Vec::<Tag> = stmt.query_map(params![workspace_id], |row| {
        let favorite: i32 = row.get(2)?;
        Ok(Tag {
            tag_id: row.get(0)?,
            name: row.get(1)?,
            favorite: favorite != 0,
            tag_group_id: row.get(3)?,
        })
    }).unwrap().map(|r| r.unwrap()).collect();
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

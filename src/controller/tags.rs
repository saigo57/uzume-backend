
use axum::{
    self,
    routing::{get, post},
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
use crate::model::file::config::Config;
use crate::model::file::tags::Tags as FileTags;
use crate::model::db::tag::Tag as DBTag;
use crate::util::BasicApiError;

#[derive(Serialize, Deserialize, ToSchema)]
struct TagsResponse {
    tags: Vec<DBTag>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TagParams {
    name: String,
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
    let tags = DBTag::get_all(&conn, workspace_id.clone()).unwrap();
    let tr = TagsResponse {
        tags,
    };
    (StatusCode::OK, Json(tr))
}

#[utoipa::path(
    post,
    path = "/api/v1/tags",
    responses(
        (status = 200, description = "All tags", body = TagsResponse)
    )
)]
async fn post_tags(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Json(body): Json<TagParams>,
) -> (StatusCode, Result<Json<DBTag>, Json<BasicApiError>>) {
    let conn = conn.lock().await;
    let tag = match DBTag::create(&conn, workspace_id.clone(), body.name) {
        Ok(tag) => tag,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };

    let config = Config::new().unwrap();
    let workspace = config.find(workspace_id.clone()).unwrap();

    let tags = DBTag::get_all(&conn, workspace_id.clone()).unwrap();
    match FileTags::save_from_db(workspace, &tags) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

    (StatusCode::CREATED, Ok(Json(tag)))
}

pub fn router(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/", get(get_tags))
        .route("/", post(post_tags))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth))
}

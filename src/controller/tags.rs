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
use crate::model::db::config::Config as DBConfig;
use crate::model::file::tags::Tags as FileTags;
use crate::model::file::writer::Writer;
use crate::model::db::tag::Tag as DBTag;
use crate::util::{ApiResponse, BasicApiError};

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
) -> (StatusCode, ApiResponse<TagsResponse>) {
    let conn = conn.lock().await;
    let tags = match DBTag::get_all(&conn, workspace_id.clone()) {
        Ok(tags) => tags,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    let tr = TagsResponse {
        tags,
    };
    (StatusCode::OK, Ok(Json(tr)))
}

#[utoipa::path(
    post,
    path = "/api/v1/tags",
    responses(
        (status = 200, description = "All tags", body = TagsResponse)
    )
)]
async fn post_tags<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    Json(body): Json<TagParams>,
) -> (StatusCode, ApiResponse<DBTag>) {
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

    let tags = match DBTag::get_all(&conn, workspace_id.clone()) {
        Ok(tags) => tags,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    match FileTags::save_from_db(&mut writer.clone() , &workspace, &tags) {
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

pub fn router<T: Writer + 'static>(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/", get(get_tags))
        .route("/", post(post_tags::<T>))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::test_util::TestUtil;

    #[tokio::test]
    async fn test_post_tags() {
        let tu = TestUtil::new().await;
        let workspace_path = Path::new(&tu.workspace_path);

        let body = Json(TagParams { name: "new_tag".to_string() });
        let (status, _result) = post_tags(
            Extension(tu.workspace_id.to_string()),
            Extension(tu.conn.clone()),
            Extension(tu.writer.clone()),
            body
        ).await;
        assert_eq!(status, StatusCode::CREATED);

        assert_eq!(tu.writer.get_path(), workspace_path.join("tags.json").to_str().unwrap());

        let tags: FileTags = serde_json::from_str(&tu.writer.get_json()).unwrap();
        assert_eq!(tags.tags.len(), 1);

        let tag = tags.tags.first().unwrap();
        assert_eq!(tag.tag_id.len(), 36);
        assert_eq!(tag.name, "new_tag");
        assert!(!tag.favorite);
        assert_eq!(tag.tag_group_id.len(), 0);
    }
}

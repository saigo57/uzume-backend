
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
async fn post_tags<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
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

    let workspace = DBConfig::find(&conn, workspace_id.clone()).unwrap().unwrap();

    let tags = DBTag::get_all(&conn, workspace_id.clone()).unwrap();
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
    use crate::schema::create_schema;
    use crate::model::file::writer::MockWriter;

    #[tokio::test]
    async fn test_post_tags() {
        let writer = MockWriter{data: Arc::new(std::sync::Mutex::new(None))};
        let conn = Connection::open_in_memory().unwrap();
        let conn = Arc::new(Mutex::new(conn));
        create_schema(conn.clone()).await.unwrap();


        let test_access_token = "test-access-token";
        let workspace_id = "12345678-xxxx-yyyy-zzzz-000000000000";
        let workspace_path = "/path/to/test.uzume";

        {
            let conn = conn.lock().await;
            conn.execute(
                "INSERT INTO auth (access_token, workspace_id) VALUES (?1, ?2)",
                [test_access_token, workspace_id],
            ).unwrap();

            conn.execute(
                "INSERT INTO config (path, workspace_id, name) VALUES (?1, ?2, ?3)",
                [workspace_path, workspace_id, "test_workspace"],
            ).unwrap();
        }

        let body = Json(TagParams { name: "new_tag".to_string() });
        let (status, _result) = post_tags(
            Extension(workspace_id.to_string()),
            Extension(conn.clone()),
            Extension(writer.clone()),
            body
        ).await;

        assert_eq!(status, StatusCode::CREATED);

        assert_eq!(writer.get_path(), "/path/to/test.uzume/tags.json");

        let tags: FileTags = serde_json::from_str(&writer.get_json()).unwrap();
        assert_eq!(tags.tags.len(), 1);

        let tag = tags.tags.first().unwrap();
        assert_eq!(tag.tag_id.len(), 36);
        assert_eq!(tag.name, "new_tag");
        assert!(!tag.favorite);
        assert_eq!(tag.tag_group_id.len(), 0);
    }
}

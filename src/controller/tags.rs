use axum::{
    self,
    routing::{get, post, patch, delete},
    http::StatusCode,
    extract::{Extension, Path},
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::db::config::Config as DBConfig;
use crate::model::file::tags::Tags as FileTags;
use crate::model::file::writer::Writer;
use crate::model::db::tag::Tag as DBTag;
use crate::util::{ApiResponse, BasicApiError};

#[derive(Serialize, Deserialize)]
struct TagsResponse {
    tags: Vec<DBTag>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TagParams {
    name: String,
}

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

async fn patch_tags<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    Path(id): Path<String>,
    Json(body): Json<TagParams>,
) -> (StatusCode, ApiResponse<DBTag>) {
    let conn = conn.lock().await;
    let mut tag = match DBTag::find(&conn, workspace_id.clone(), id.clone()) {
        Ok(tag_opt) => {
            match tag_opt {
                Some(tag) => tag,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Err(Json(BasicApiError { error_message: format!("tag({id}) not found.") }))
                    );
                }
            }
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    
    tag.name = body.name.clone();
    match tag.save(&conn) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

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

    (StatusCode::OK, Ok(Json(tag)))
}

async fn delete_tags<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    Path(id): Path<String>,
) -> (StatusCode, ApiResponse<DBTag>) {
    let conn = conn.lock().await;
    let tag = match DBTag::find(&conn, workspace_id.clone(), id.clone()) {
        Ok(tag_opt) => {
            match tag_opt {
                Some(tag) => tag,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Err(Json(BasicApiError { error_message: format!("tag({id}) not found.") }))
                    );
                }
            }
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    
    match tag.delete(&conn) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

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

    (StatusCode::OK, Ok(Json(tag)))
}

async fn post_tags_favorite<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    Path(id): Path<String>,
) -> (StatusCode, ApiResponse<DBTag>) {
    let conn = conn.lock().await;
    let mut tag = match DBTag::find(&conn, workspace_id.clone(), id.clone()) {
        Ok(tag_opt) => {
            match tag_opt {
                Some(tag) => tag,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Err(Json(BasicApiError { error_message: format!("tag({id}) not found.") }))
                    );
                }
            }
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    
    tag.favorite = true;
    match tag.save(&conn) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

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

    (StatusCode::OK, Ok(Json(tag)))
}

async fn delete_tags_favorite<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    Path(id): Path<String>,
) -> (StatusCode, ApiResponse<DBTag>) {
    let conn = conn.lock().await;
    let mut tag = match DBTag::find(&conn, workspace_id.clone(), id.clone()) {
        Ok(tag_opt) => {
            match tag_opt {
                Some(tag) => tag,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Err(Json(BasicApiError { error_message: format!("tag({id}) not found.") }))
                    );
                }
            }
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    
    tag.favorite = false;
    match tag.save(&conn) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

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

    (StatusCode::OK, Ok(Json(tag)))
}

pub fn router<T: Writer + 'static>(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/", get(get_tags))
        .route("/", post(post_tags::<T>))
        .route("/:id", patch(patch_tags::<T>))
        .route("/:id", delete(delete_tags::<T>))
        .route("/:id/favorite", post(post_tags_favorite::<T>))
        .route("/:id/favorite", delete(delete_tags_favorite::<T>))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::test_util::TestUtil;

    mod test_get_tags {
        use super::*;

        #[tokio::test]
        async fn test_get_tags() {
            let tu = TestUtil::new().await;

            let mut tags = Vec::new();
            {
                let conn = tu.conn.lock().await;
                let tag1 = DBTag::create(&conn, tu.workspace_id.clone(), "tag1".to_string()).unwrap();
                let mut tag2 = DBTag::create(&conn, tu.workspace_id.clone(), "tag2".to_string()).unwrap();
                tag2.favorite = true;
                tag2.save(&conn).unwrap();

                tags.push(tag1);
                tags.push(tag2);
            }

            let (status, result) = get_tags(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone())
            ).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.tags.len(), 2);
            assert_eq!(result.tags[0].workspace_id, tu.workspace_id);
            assert_eq!(result.tags[0].tag_id, tags[0].tag_id);
            assert_eq!(result.tags[0].name, tags[0].name);
            assert_eq!(result.tags[0].favorite, tags[0].favorite);
            assert_eq!(result.tags[1].workspace_id, tu.workspace_id);
            assert_eq!(result.tags[1].tag_id, tags[1].tag_id);
            assert_eq!(result.tags[1].name, tags[1].name);
            assert_eq!(result.tags[1].favorite, tags[1].favorite);
        }
    }
    
    mod test_post_tags {
        use super::*;

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
            
            {
                let conn = tu.conn.lock().await;
                let db_tags = DBTag::get_all(&conn, tu.workspace_id.clone()).unwrap();
                assert_eq!(db_tags.len(), 1);
                assert_eq!(db_tags[0].workspace_id, tu.workspace_id);
                assert_eq!(db_tags[0].tag_id.len(), 36);
                assert_eq!(db_tags[0].name, "new_tag");
                assert!(!db_tags[0].favorite);
                assert_eq!(db_tags[0].tag_group_id, "");
            }

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].path, workspace_path.join("tags.json").to_str().unwrap());

            let tags: FileTags = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(tags.tags.len(), 1);

            let tag = tags.tags.first().unwrap();
            assert_eq!(tag.tag_id.len(), 36);
            assert_eq!(tag.name, "new_tag");
            assert!(!tag.favorite);
            assert_eq!(tag.tag_group_id.len(), 0);
        }
    }

    mod test_patch_tags {
        use super::*;
        
        async fn setup_tags(tu: &TestUtil) -> Vec<DBTag> {
            let mut tags = Vec::new();
            {
                let conn = tu.conn.lock().await;
                let tag1 = DBTag::create(&conn, tu.workspace_id.clone(), "tag1".to_string()).unwrap();
                let mut tag2 = DBTag::create(&conn, tu.workspace_id.clone(), "tag2".to_string()).unwrap();
                tag2.favorite = true;
                tag2.save(&conn).unwrap();

                tags.push(tag1);
                tags.push(tag2);
            }
            tags
        }

        #[tokio::test]
        async fn test_patch_tags() {
            let tu = TestUtil::new().await;
            let workspace_path = Path::new(&tu.workspace_path);
            let tags = setup_tags(&tu).await;

            let body = Json(TagParams { name: "update_tag_name".to_string() });
            let (status, result) = patch_tags(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path(tags[0].tag_id.clone()),
                body
            ).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.workspace_id, tu.workspace_id);
            assert_eq!(result.tag_id, tags[0].tag_id);
            assert_eq!(result.name, "update_tag_name".to_string());
            assert_eq!(result.favorite, tags[0].favorite);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].path, workspace_path.join("tags.json").to_str().unwrap());

            let file_result: FileTags = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(file_result.tags.len(), 2);

            assert_eq!(file_result.tags[0].tag_id, tags[0].tag_id);
            assert_eq!(file_result.tags[0].name, "update_tag_name");
            assert_eq!(file_result.tags[0].favorite, tags[0].favorite);
            assert_eq!(file_result.tags[0].tag_group_id, tags[0].tag_group_id);

            assert_eq!(file_result.tags[1].tag_id, tags[1].tag_id);
            assert_eq!(file_result.tags[1].name, tags[1].name);
            assert_eq!(file_result.tags[1].favorite, tags[1].favorite);
            assert_eq!(file_result.tags[1].tag_group_id, tags[1].tag_group_id);
        }

        #[tokio::test]
        async fn test_patch_tags_not_found() {
            let tu = TestUtil::new().await;
            setup_tags(&tu).await;

            let body = Json(TagParams { name: "update_tag_name".to_string() });
            let (status, _) = patch_tags(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path("invalid_tag_id".to_string()),
                body
            ).await;
            assert_eq!(status, StatusCode::NOT_FOUND);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 0);
        }
    }

    mod test_delete_tags {
        use super::*;
        
        async fn setup_tags(tu: &TestUtil) -> Vec<DBTag> {
            let mut tags = Vec::new();
            {
                let conn = tu.conn.lock().await;
                let tag1 = DBTag::create(&conn, tu.workspace_id.clone(), "tag1".to_string()).unwrap();
                let mut tag2 = DBTag::create(&conn, tu.workspace_id.clone(), "tag2".to_string()).unwrap();
                tag2.favorite = true;
                tag2.save(&conn).unwrap();

                tags.push(tag1);
                tags.push(tag2);
            }
            tags
        }

        #[tokio::test]
        async fn test_patch_tags() {
            let tu = TestUtil::new().await;
            let workspace_path = Path::new(&tu.workspace_path);
            let tags = setup_tags(&tu).await;

            let (status, result) = delete_tags(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path(tags[0].tag_id.clone()),
            ).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.workspace_id, tu.workspace_id);
            assert_eq!(result.tag_id, tags[0].tag_id);
            assert_eq!(result.name, tags[0].name);
            assert_eq!(result.favorite, tags[0].favorite);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].path, workspace_path.join("tags.json").to_str().unwrap());

            let file_result: FileTags = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(file_result.tags.len(), 1);

            assert_eq!(file_result.tags[0].tag_id, tags[1].tag_id);
            assert_eq!(file_result.tags[0].name, tags[1].name);
            assert_eq!(file_result.tags[0].favorite, tags[1].favorite);
            assert_eq!(file_result.tags[0].tag_group_id, tags[1].tag_group_id);
        }

        #[tokio::test]
        async fn test_delete_tags_not_found() {
            let tu = TestUtil::new().await;
            setup_tags(&tu).await;

            let (status, _) = delete_tags(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path("invalid_tag_id".to_string()),
            ).await;
            assert_eq!(status, StatusCode::NOT_FOUND);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 0);
        }
    }

    mod test_tag_favorite {
        use super::*;
        
        async fn setup_tags(tu: &TestUtil) -> Vec<DBTag> {
            let mut tags = Vec::new();
            {
                let conn = tu.conn.lock().await;
                let tag1 = DBTag::create(&conn, tu.workspace_id.clone(), "tag1".to_string()).unwrap();
                let mut tag2 = DBTag::create(&conn, tu.workspace_id.clone(), "tag2".to_string()).unwrap();
                tag2.favorite = true;
                tag2.save(&conn).unwrap();

                tags.push(tag1);
                tags.push(tag2);
            }
            tags
        }

        #[tokio::test]
        async fn test_post_tags_favorite() {
            let tu = TestUtil::new().await;
            let workspace_path = Path::new(&tu.workspace_path);
            let tags = setup_tags(&tu).await;

            assert!(!tags[0].favorite);
            let (status, result) = post_tags_favorite(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path(tags[0].tag_id.clone()),
            ).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.workspace_id, tu.workspace_id);
            assert_eq!(result.tag_id, tags[0].tag_id);
            assert_eq!(result.name, tags[0].name);
            assert_eq!(result.favorite, !tags[0].favorite);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].path, workspace_path.join("tags.json").to_str().unwrap());

            let file_result: FileTags = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(file_result.tags.len(), 2);

            assert_eq!(file_result.tags[0].tag_id, tags[0].tag_id);
            assert_eq!(file_result.tags[0].name, tags[0].name);
            assert_eq!(file_result.tags[0].favorite, !tags[0].favorite);
            assert_eq!(file_result.tags[0].tag_group_id, tags[0].tag_group_id);

            assert_eq!(file_result.tags[1].tag_id, tags[1].tag_id);
            assert_eq!(file_result.tags[1].name, tags[1].name);
            assert_eq!(file_result.tags[1].favorite, tags[1].favorite);
            assert_eq!(file_result.tags[1].tag_group_id, tags[1].tag_group_id);
        }

        #[tokio::test]
        async fn test_delete_tags_favorite() {
            let tu = TestUtil::new().await;
            let workspace_path = Path::new(&tu.workspace_path);
            let tags = setup_tags(&tu).await;

            assert!(tags[1].favorite);
            let (status, result) = delete_tags_favorite(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path(tags[1].tag_id.clone()),
            ).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.workspace_id, tu.workspace_id);
            assert_eq!(result.tag_id, tags[1].tag_id);
            assert_eq!(result.name, tags[1].name);
            assert_eq!(result.favorite, !tags[1].favorite);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].path, workspace_path.join("tags.json").to_str().unwrap());

            let file_result: FileTags = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(file_result.tags.len(), 2);

            assert_eq!(file_result.tags[0].tag_id, tags[0].tag_id);
            assert_eq!(file_result.tags[0].name, tags[0].name);
            assert_eq!(file_result.tags[0].favorite, tags[0].favorite);
            assert_eq!(file_result.tags[0].tag_group_id, tags[0].tag_group_id);

            assert_eq!(file_result.tags[1].tag_id, tags[1].tag_id);
            assert_eq!(file_result.tags[1].name, tags[1].name);
            assert_eq!(file_result.tags[1].favorite, !tags[1].favorite);
            assert_eq!(file_result.tags[1].tag_group_id, tags[1].tag_group_id);
        }

        #[tokio::test]
        async fn test_delete_tags_favorite_not_found() {
            let tu = TestUtil::new().await;
            setup_tags(&tu).await;

            let (status, _) = delete_tags_favorite(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                Path("invalid_tag_id".to_string()),
            ).await;
            assert_eq!(status, StatusCode::NOT_FOUND);

            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 0);
        }
    }
}

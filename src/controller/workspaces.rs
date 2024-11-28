use axum::{
    self,
    routing::{get, post, patch, delete},
    http::StatusCode,
    extract::{Extension, Multipart},
    response::{Response, IntoResponse},
    body::Body,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::file::config::Config as FileConfig;
use crate::model::file::workspace_info::WorkspaceInfo as FileWorkspaceInfo;
use crate::model::file::workspace::Workspace as FileWorkspace;
use crate::model::db::config::Config as DBConfig;
use crate::model::db::auth::Auth as DBAuth;
use crate::model::file::writer::Writer;
use crate::util::{ApiResponse, BasicApiError};
use crate::multipart_params::MultipartParams;

#[derive(Debug, Serialize)]
struct WorkspaceResponse {
    workspace_list: Vec<FileWorkspaceInfo>,
}

#[derive(Debug, Serialize)]
struct LoginInfoResponse {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspacePatchParams {
    name: String,
}

#[derive(Deserialize)]
struct LoginWorkspaceParams {
    workspace_id: String,
}

#[derive(Deserialize)]
struct IconMultipartBody {
    #[allow(dead_code)]
    icon: Vec<u8>,
}

async fn get_workspaces(
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> (StatusCode, ApiResponse<WorkspaceResponse>) {
    let conn = conn.lock().await;
    let workspaces = match DBConfig::get_workspaces(&conn) {
        Ok(config) => config,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    let res = WorkspaceResponse {
        workspace_list: workspaces,
    };
    (StatusCode::OK, Ok(Json(res)))
}

async fn patch_workspaces<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
    Json(body): Json<WorkspacePatchParams>,
) -> (StatusCode, ApiResponse<()>) {
    let conn = conn.lock().await;
    
    match DBConfig::update(&conn, workspace_id.clone(), body.name.clone()) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };

    let workspaces = match DBConfig::get_workspaces(&conn) {
        Ok(config) => config,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    
    match FileConfig::save_from_db(&mut writer.clone(), &workspaces) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

    (StatusCode::NO_CONTENT, Ok(Json(())))
}

async fn delete_workspaces<T: Writer>(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Extension(writer): Extension<T>,
) -> (StatusCode, ApiResponse<()>) {
    let conn = conn.lock().await;
    
    match DBConfig::delete(&conn, workspace_id.clone()) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };

    let workspaces = match DBConfig::get_workspaces(&conn) {
        Ok(config) => config,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };

    match FileConfig::save_from_db(&mut writer.clone(), &workspaces) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

    (StatusCode::NO_CONTENT, Ok(Json(())))
}

async fn login_workspace(
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Json(body): Json<LoginWorkspaceParams>,
) -> (StatusCode, ApiResponse<LoginInfoResponse>) {
    let conn = conn.lock().await;
    
    match DBConfig::find(&conn, body.workspace_id.clone()) {
        Ok(Some(_)) => {},
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Err(Json(BasicApiError { error_message: "Workspace not found".to_string() }))
            );
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };

    let auth = DBAuth::generate(body.workspace_id.clone());
    match auth.save(&conn) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

    (StatusCode::OK, Ok(Json(LoginInfoResponse { access_token: auth.access_token })))
}

async fn get_workspaces_icon(
    Extension(workspace_id): Extension<String>,
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> Response {
    let conn = conn.lock().await;
    
    match FileWorkspace::get_icon_image(&conn, &workspace_id) {
        Ok(image) => {
            axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", format!("image/{}", image.ext))
                .body(Body::from(image.data))
                .unwrap()
        },
        Err(err) => {
            // まだアイコンが設定されていない場合は404を返す
            log::info!("get icon image: {}", err);
            return (
                StatusCode::NOT_FOUND,
                ()
            )
            .into_response();
        },
    }
}

async fn post_workspaces_icon<T: Writer>(
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
    
    let icon_field = &multipart_params.files[0];
    if icon_field.param_name != "icon" {
        log::error!("Invalid parameter name: {}", icon_field.param_name);
        return (
            StatusCode::BAD_REQUEST,
            Err(Json(BasicApiError { error_message: "Invalid parameter name".to_string() }))
        );
    }

    let path = std::path::Path::new(&icon_field.file_name);
    let ext_str = match path.extension() {
        Some(ext) => ext.to_str(),
        None => None,
    };
    let ext_str = match ext_str {
        Some(ext) => ext,
        None => {
            log::error!("get extension error.");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: "get extension error.".to_string() }))
            );
        },
    };

    match FileWorkspace::save_icon(&conn, &mut writer.clone(), &workspace_id, &icon_field.data, ext_str) {
        Ok(_) => {},
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

pub fn router<T: Writer + 'static>(conn: Arc<Mutex<Connection>>) -> Router {
    let noauth_endpoints = Router::new()
        .route("/", get(get_workspaces))
        .route("/login", post(login_workspace));
    let auth_endpoints = Router::new()
        .route("/", patch(patch_workspaces::<T>))
        .route("/", delete(delete_workspaces::<T>))
        .route("/icon", get(get_workspaces_icon))
        .route("/icon", post(post_workspaces_icon::<T>))
        .route_layer(axum::middleware::from_fn_with_state(conn.clone(), auth));

    Router::new()
        .nest("/", noauth_endpoints)
        .nest("/", auth_endpoints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TestUtil;

    mod test_get_workspace {
        use super::*;

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;
            
            {
                let conn = tu.conn.lock().await;
                conn.execute(
                    "INSERT INTO config (path, workspace_id, name) VALUES (?1, ?2, ?3)",
                    ["/path/to/hoge_workspac.uzume", "12345678-xxxx-hoge-zzzz-000000000000", "hoge workspace"],
                ).unwrap();
            }

            let (status, result) = get_workspaces(Extension(tu.conn.clone())).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.0.workspace_list.len(), 2);
            assert_eq!(result.0.workspace_list[0].path, tu.workspace_path);
            assert_eq!(result.0.workspace_list[0].workspace_id, tu.workspace_id);
            assert_eq!(result.0.workspace_list[0].name, tu.workspace_name);
            assert_eq!(result.0.workspace_list[1].path, "/path/to/hoge_workspac.uzume");
            assert_eq!(result.0.workspace_list[1].workspace_id, "12345678-xxxx-hoge-zzzz-000000000000");
            assert_eq!(result.0.workspace_list[1].name, "hoge workspace");
        }
    }

    mod test_patch_workspace {
        use super::*;

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;
            
            {
                let conn = tu.conn.lock().await;
                conn.execute(
                    "INSERT INTO config (path, workspace_id, name) VALUES (?1, ?2, ?3)",
                    ["/path/to/hoge_workspac.uzume", "12345678-xxxx-hoge-zzzz-000000000000", "hoge workspace"],
                ).unwrap();
            }

            let body = Json(WorkspacePatchParams { name: "new_workspace name".to_string() });
            let (status, _result) = patch_workspaces(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                body
            ).await;
            assert_eq!(status, StatusCode::NO_CONTENT);

            {
                let conn = tu.conn.lock().await;
                let workspace = DBConfig::find(&conn, tu.workspace_id.clone()).unwrap().unwrap();
                assert_eq!(workspace.path, tu.workspace_path);
                assert_eq!(workspace.workspace_id, tu.workspace_id);
                assert_eq!(workspace.name, "new_workspace name");
            }
            
            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            let config: FileConfig = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(config.workspace_list.len(), 2);
            assert_eq!(config.workspace_list[0].path, tu.workspace_path);
            assert_eq!(config.workspace_list[0].workspace_id, tu.workspace_id);
            assert_eq!(config.workspace_list[0].name, "new_workspace name");
            assert_eq!(config.workspace_list[1].path, "/path/to/hoge_workspac.uzume");
            assert_eq!(config.workspace_list[1].workspace_id, "12345678-xxxx-hoge-zzzz-000000000000");
            assert_eq!(config.workspace_list[1].name, "hoge workspace");
        }
    }

    mod test_delete_workspace {
        use super::*;

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;
            
            {
                let conn = tu.conn.lock().await;
                conn.execute(
                    "INSERT INTO config (path, workspace_id, name) VALUES (?1, ?2, ?3)",
                    ["/path/to/hoge_workspac.uzume", "12345678-xxxx-hoge-zzzz-000000000000", "hoge workspace"],
                ).unwrap();
            }

            let (status, _result) = delete_workspaces(
                Extension(tu.workspace_id.to_string()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
            ).await;
            assert_eq!(status, StatusCode::NO_CONTENT);

            {
                let conn = tu.conn.lock().await;
                let workspace = DBConfig::find(&conn, tu.workspace_id.clone()).unwrap();
                assert!(workspace.is_none());
            }
            
            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            let config: FileConfig = serde_json::from_str(&history[0].data).unwrap();
            assert_eq!(config.workspace_list.len(), 1);
            assert_eq!(config.workspace_list[0].path, "/path/to/hoge_workspac.uzume");
            assert_eq!(config.workspace_list[0].workspace_id, "12345678-xxxx-hoge-zzzz-000000000000");
            assert_eq!(config.workspace_list[0].name, "hoge workspace");
        }
    }
    
    mod test_login_workspace {
        use super::*;

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;
            
            {
                let conn = tu.conn.lock().await;
                assert_eq!(DBAuth::count(&conn, tu.workspace_id.clone()).unwrap(), 1);
            }

            let body = Json(LoginWorkspaceParams { workspace_id: tu.workspace_id.clone() });
            let (status, result) = login_workspace(Extension(tu.conn.clone()), body).await;
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.0.access_token.len(), 36);

            {
                let conn = tu.conn.lock().await;
                assert_eq!(DBAuth::count(&conn, tu.workspace_id.clone()).unwrap(), 2);
            }
        }

        #[tokio::test]
        async fn test_fail() {
            let tu = TestUtil::new().await;

            {
                let conn = tu.conn.lock().await;
                assert_eq!(DBAuth::count(&conn, tu.workspace_id.clone()).unwrap(), 1);
            }

            let body = Json(LoginWorkspaceParams { workspace_id: "invalid_workspace_id".to_string() });
            let (status, _result) = login_workspace(Extension(tu.conn.clone()), body).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);

            {
                let conn = tu.conn.lock().await;
                assert_eq!(DBAuth::count(&conn, tu.workspace_id.clone()).unwrap(), 1);
            }
        }
    }
    
    
    mod test_post_workspace_icon {
        use super::*;
        use hyper;
        use axum::body::Body;
        use axum::extract::Request;
        use axum::extract::FromRequest;

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;
            
            {
                let conn = tu.conn.lock().await;
                assert_eq!(DBAuth::count(&conn, tu.workspace_id.clone()).unwrap(), 1);
            }

            let boundary = "testboundary";
            let body = format!(
                "--{}\r\n\
                 Content-Disposition: form-data; name=\"icon\"; filename=\"test-icon.png\"\r\n\
                 Content-Type: image/png\r\n\r\n\
                 {}\r\n\
                 --{}--\r\n",
                boundary, "dummy image data", boundary
            );
            let request = Request::builder()
                .header(
                    hyper::header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap();
            let multipart = Multipart::from_request(request, &()).await.unwrap();

            let (status, _result) = post_workspaces_icon(
                Extension(tu.workspace_id.clone()),
                Extension(tu.conn.clone()),
                Extension(tu.writer.clone()),
                multipart,
            ).await;
            
            assert_eq!(status, StatusCode::CREATED);
            let history = tu.writer.history.lock().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].path, format!("{}/icon.png", tu.workspace_path));
            assert_eq!(history[0].data, "dummy image data");
        }
    }
}

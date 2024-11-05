use axum::{
    self,
    routing::{get, post, patch},
    http::StatusCode,
    extract::Extension,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use utoipa::{OpenApi, ToSchema, IntoParams};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::file::config::Config as FileConfig;
use crate::model::file::workspace_info::WorkspaceInfo as FileWorkspaceInfo;
use crate::model::db::config::Config as DBConfig;
use crate::model::db::auth::Auth as DBAuth;
use crate::util::{ApiResponse, BasicApiError};

#[derive(Debug, Serialize, ToSchema)]
struct WorkspaceResponse {
    workspace_list: Vec<FileWorkspaceInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
struct LoginInfoResponse {
    access_token: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
struct LoginWorkspaceParams {
    workspace_id: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, description = "All workspaces", body = WorkspaceResponse)
    ),
    tag="workspace",
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, description = "All workspaces", body = Config)
    ),
    tag="workspace",
)]
async fn patch_workspaces(
    Extension(workspace_id): Extension<String>,
) -> (StatusCode, ApiResponse<FileConfig>) {
    println!("workspace_id: {}", workspace_id);
    let config = match FileConfig::new() {
        Ok(config) => config,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    };
    (StatusCode::OK, Ok(Json(config)))
}

#[utoipa::path(
    post,
    path = "/api/v1/workspaces/login",
    params(LoginWorkspaceParams),
    responses(
        (status = 200, description = "Login success", body = LoginInfoResponse),
        (status = 400, description = "Login failed", body = BasicApiError),
    ),
    tag="workspace",
)]
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

#[derive(OpenApi)]
#[openapi(
    paths(
        get_workspaces,
        // post_workspaces,
        // patch_workspaces,
        // delete_workspace,
        // get_workspace_icon
        // post_workspaces_icon,
        login_workspace,
        // post_workspace_add,
    ),
    components(
        schemas(
            WorkspaceResponse,
            FileWorkspaceInfo,
            LoginWorkspaceParams,
            LoginInfoResponse,
            BasicApiError,
        ),
    ),
)]
pub struct ApiDoc;

pub fn router(conn: Arc<Mutex<Connection>>) -> Router {
    let noauth_endpoints = Router::new()
        .route("/", get(get_workspaces))
        .route("/login", post(login_workspace));
    let auth_endpoints = Router::new()
        .route("/", patch(patch_workspaces))
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
            let (status, result) = login_workspace(Extension(tu.conn.clone()), body).await;
            print!("status: {:?}, result: {:?}", status, result);
            assert_eq!(status, StatusCode::BAD_REQUEST);

            {
                let conn = tu.conn.lock().await;
                assert_eq!(DBAuth::count(&conn, tu.workspace_id.clone()).unwrap(), 1);
            }
        }
    }
}

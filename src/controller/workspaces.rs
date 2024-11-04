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
use crate::model::db::config::Config as DBConfig;
use crate::model::db::auth::Auth as DBAuth;
use crate::util::{ApiResponse, BasicApiError};

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
        (status = 200, description = "All workspaces", body = Config)
    )
)]
async fn get_workspaces() -> (StatusCode, ApiResponse<FileConfig>) {
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
    path = "/api/v1/workspaces",
    responses(
        (status = 200, description = "All workspaces", body = Config)
    )
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
    )
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
        login_workspace,
    ),
    components(
        schemas(
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
    
    mod test_login_workspace {
        use super::*;

        #[tokio::test]
        async fn test_success() {
            let tu = TestUtil::new().await;

            let body = Json(LoginWorkspaceParams { workspace_id: tu.workspace_id.clone() });
            let (status, result) = login_workspace(Extension(tu.conn.clone()), body).await;
            print!("status: {:?}, result: {:?}", status, result);
            assert_eq!(status, StatusCode::OK);
            let result = result.unwrap();
            assert_eq!(result.0.access_token.len(), 36);
        }

        #[tokio::test]
        async fn test_fail() {
            let tu = TestUtil::new().await;

            let body = Json(LoginWorkspaceParams { workspace_id: "invalid_workspace_id".to_string() });
            let (status, result) = login_workspace(Extension(tu.conn.clone()), body).await;
            print!("status: {:?}, result: {:?}", status, result);
            assert_eq!(status, StatusCode::BAD_REQUEST);
        }
    }
}

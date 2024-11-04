
use axum::{
    self,
    routing::{get, post, patch},
    http::StatusCode,
    extract::Extension,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use utoipa::OpenApi;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::controller::middleware::auth;
use crate::model::file::config::Config;
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::db::auth::Auth as DBAuth;
use crate::util::{ApiResponse, BasicApiError};

#[derive(Debug, Serialize)]
struct LoginInfoResponse {
    access_token: String,
}

#[derive(Deserialize)]
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
async fn get_workspaces() -> (StatusCode, ApiResponse<Config>) {
    let config = match Config::new() {
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
) -> (StatusCode, ApiResponse<Config>) {
    println!("workspace_id: {}", workspace_id);
    let config = match Config::new() {
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
    responses(
        (status = 200, description = "Login success", body = LoginInfo),
        (status = 400, description = "Login failed", body = BasicApiError),
    )
)]
async fn login_workspace(
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Json(body): Json<LoginWorkspaceParams>,
) -> (StatusCode, ApiResponse<LoginInfoResponse>) {
    let conn = conn.lock().await;
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
    ),
    components(
        schemas(
            Config,
            WorkspaceInfo,
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
    use crate::schema::create_schema;

    #[tokio::test]
    async fn test_post_login_workspace() {
        let conn = Connection::open_in_memory().unwrap();
        let conn = Arc::new(Mutex::new(conn));
        create_schema(conn.clone()).await.unwrap();

        let body = Json(LoginWorkspaceParams { workspace_id: "test_workspace_id".to_string() });
        let (status, result) = login_workspace(Extension(conn.clone()), body).await;
        print!("status: {:?}, result: {:?}", status, result);
        assert_eq!(status, StatusCode::OK);
        let result = result.unwrap();
        assert_eq!(result.0.access_token.len(), 36);
    }
}

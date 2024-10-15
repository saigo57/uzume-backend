
use uuid::Uuid;
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
use crate::model::config::Config;
use crate::model::workspace_info::WorkspaceInfo;

// TODO: unwrap周りと適切に処理して、model化する
// TODO: 自動テストを書く
//trait JsonModel: Sized + DeserializeOwned + Serialize {
    //fn file_path(&self) -> String;

    //async fn save(&self) -> Result<(), std::io::Error> {
        //let json = serde_json::to_string_pretty(&self).unwrap();
        //let mut file = File::create(self.file_path()).await?;
        //file.write_all(json.as_bytes()).await?;
        //Ok(())
    //}

    //fn new(file_path: &str) -> Result<Self, std::io::Error> {
        //let json_file = std::fs::File::open(file_path).unwrap();
        //let reader = std::io::BufReader::new(json_file);
        //let config = serde_json::from_reader(reader).unwrap();
        //Ok(config)
    //}
//}



#[derive(Serialize)]
struct LoginInfo {
    access_token: String,
}

#[derive(Serialize)]
struct BasicApiError {
    error_message: String,
}

//impl JsonModel for Config {
    //fn file_path(&self) -> String {
        //Self::FILE_PATH.to_string()
    //}
//}


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
async fn get_workspaces() -> (StatusCode, Json<Config>) {
    let config = Config::new().unwrap();
    (StatusCode::OK, Json(config))
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
) -> (StatusCode, Json<Config>) {
    println!("workspace_id: {}", workspace_id);
    let config = Config::new().unwrap();
    (StatusCode::OK, Json(config))
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
) -> (StatusCode, Result<Json<LoginInfo>, Json<BasicApiError>>) {
    let conn = conn.lock().await;
    let access_token = Uuid::new_v4().to_string();
    match conn.execute(
        "INSERT INTO auth (access_token, workspace_id) VALUES (?1, ?2)",
        [access_token.clone(), body.workspace_id],
    ) {
        Ok(_) => {},
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(Json(BasicApiError { error_message: err.to_string() }))
            );
        }
    }

    (StatusCode::OK, Ok(Json(LoginInfo { access_token })))
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

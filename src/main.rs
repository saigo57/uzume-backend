use std::env;
use uuid::Uuid;
use axum::{
    routing::get,
    routing::post,
    http::StatusCode,
    extract::DefaultBodyLimit,
    extract::Extension,
    Json,
    Router,
};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

const DEFAULT_PORT: u16 = 22113;

// TODO: unwrap周りと適切に処理して、model化する
// TODO: 自動テストを書く
trait JsonModel: Sized + DeserializeOwned + Serialize {
    fn file_path(&self) -> String;

    async fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(self.file_path()).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }

    fn new(file_path: &str) -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(file_path).unwrap();
        let reader = std::io::BufReader::new(json_file);
        let config = serde_json::from_reader(reader).unwrap();
        Ok(config)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
struct WorkspaceInfo {
    #[schema(example = r"C:\Users\user\workspace")]
    path: String,

    #[schema(example = "a0b257bb-b7c6-46f3-b27c-31f8ce1c3703")]
    workspace_id: String,

    #[schema(example = "ワークスペース名")]
    name: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct Config {
    workspace_list: Vec<WorkspaceInfo>,
}

impl Config {
    const FILE_PATH: &'static str = "./config.json";

    fn new() -> Result<Self, std::io::Error> {
        let json_file = std::fs::File::open(Self::FILE_PATH).unwrap();
        let reader = std::io::BufReader::new(json_file);
        let config = serde_json::from_reader(reader).unwrap();
        Ok(config)
    }

}

#[derive(Serialize)]
struct LoginInfo {
    access_token: String,
}
#[derive(Serialize)]
struct BasicApiError {
    error_message: String,
}

impl JsonModel for Config {
    fn file_path(&self) -> String {
        Self::FILE_PATH.to_string()
    }
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

#[derive(Deserialize)]
struct LoginWorkspaceParams {
    workspace_id: String,
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
) -> (StatusCode, Json<LoginInfo>) {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT access_token, workspace_id FROM auth").unwrap();
    stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).unwrap().for_each(|result| {
        let (access_token, workspace_id): (String, String) = result.unwrap();
        println!("access_token: {}, workspace_id: {}", access_token, workspace_id);
    });
    let access_token = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO auth (access_token, workspace_id) VALUES (?1, ?2)",
        [access_token.clone(), body.workspace_id],
    );


    (StatusCode::OK, Json(LoginInfo { access_token: access_token }))

}

fn parse_args(args: &[String]) -> Result<u16, String> {
    if args.len() == 1 {
        return Ok(DEFAULT_PORT);
    }

    if args.len() == 2 {
        let port_str: &str = &args[1];
        return match port_str.parse::<u16>() {
            Ok(port) => Ok(port),
            Err(_) => Err(format!("invalid port: {}", port_str)),
        };
    }

    Err(format!("invalid arg num: {}", args.len()))
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
struct ApiDoc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let port = match parse_args(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    println!("port: {}", port);

    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE auth (
            access_token TEXT,
            workspace_id TEXT
        )",
        [],
    ).unwrap();
    let workspace_router = Router::new()
        .route("/api/v1/workspaces", get(get_workspaces))
        .route("/api/v1/workspaces/login", post(login_workspace));
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1/workspace", workspace_router)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(Extension(Arc::new(Mutex::new(conn))));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use std::env;
use axum::{
    routing::get,
    http::StatusCode,
    extract::DefaultBodyLimit,
    Json,
    Router,
};
use serde::{Serialize, Deserialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;

const DEFAULT_PORT: u16 = 22113;

// TODO: unwrap周りと適切に処理して、model化する
// TODO: 自動テストを書く
trait JsonModel {
    const FILE_PATH: &'static str;
    async fn save(&self) -> Result<(), std::io::Error>;
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

impl JsonModel for Config {
    const FILE_PATH: &'static str = "./config.json";

    async fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(Self::FILE_PATH).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct HelloRequest {
    name: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, description = "All workspaces", body = Config)
    )
)]
async fn get_workspaces() -> (StatusCode, Json<Config>) {
    let json = tokio::fs::read_to_string("./config.json").await.unwrap();
    let config = serde_json::from_str(&json).unwrap();
 
    (StatusCode::OK, Json(config))
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

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/v1/workspaces", get(get_workspaces))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

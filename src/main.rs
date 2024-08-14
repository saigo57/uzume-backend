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

mod controller;
mod schema;

const DEFAULT_PORT: u16 = 22113;


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
    let v1_api_router = Router::new()
        .nest("/workspaces", controller::workspaces::router());
    match schema::create_schema(&conn) {
        Ok(_) => {
            println!("schema created.");
        },
        Err(e) => {
            eprintln!("schema create error!");
            eprintln!("{}", e);
            return;
        }
    }
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", controller::workspaces::ApiDoc::openapi()))
        .nest("/api/v1", v1_api_router)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(Extension(Arc::new(Mutex::new(conn))));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

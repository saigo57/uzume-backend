use std::env;
use axum::response::Response;
use uuid::Uuid;
use axum::{
    routing::get,
    routing::post,
    http::StatusCode,
    extract::{Extension, DefaultBodyLimit, Request, State},
    Json,
    Router,
    middleware::{self, Next},
};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use rusqlite::{Connection, params};
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

async fn auth(
    State(conn): State<Arc<Mutex<Connection>>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = match req.headers().get("Authorization").cloned() {
        Some(h) => h,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let auth_header_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let prefix = "Basic ";
    if !auth_header_str.starts_with(prefix) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let v: Vec<&str> = auth_header_str[prefix.len()..].split(':').collect();
    if v.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let workspace_id = v[0];
    let access_token = v[1];

    {
        // このconnのスコープを早めに閉じないと、next.runでロックが解除されないなどの影響？でビルドエラーになる
        let conn = conn.lock().await;
        let mut stmt = conn.prepare("SELECT 1 FROM auth WHERE workspace_id = ?1 AND access_token = ?2").unwrap();
        
        let rows = stmt.query_map(params![workspace_id, access_token], |_row| {
            Ok(1)
        }).unwrap();

        // 上の組み合わせが見つからなかったら認証エラー
        if rows.count() == 0 {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    req.extensions_mut().insert(workspace_id.to_string());
    Ok(next.run(req).await)
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
    let conn = Arc::new(Mutex::new(conn));
    let v1_api_router = Router::new()
        .nest("/workspaces", controller::workspaces::router())
        .route_layer(middleware::from_fn_with_state(conn.clone(), auth));
    match schema::create_schema(conn.clone()).await {
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
        .layer(Extension(conn));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use axum::response::Response;
use axum::{
    http::StatusCode,
    extract::{Request, State},
    middleware::Next,
};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use base64::prelude::*;
use crate::model::db::auth::Auth as DBAuth;

fn decode_str(auth_header: &str) -> Option<String> {
    let bytes = BASE64_STANDARD.decode(auth_header).ok()?;
    let s = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return None,
    };

    Some(s)
}

pub async fn auth(
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

    let decoded_payload = match decode_str(&auth_header_str[prefix.len()..]) {
        Some(s) => s,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let v: Vec<&str> = decoded_payload.split(':').collect();
    if v.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let workspace_id = v[0];
    let access_token = v[1];

    {
        // このconnのスコープを早めに閉じないと、next.runでロックが解除されないなどの影響？でビルドエラーになる
        let conn = conn.lock().await;
        match DBAuth::is_authed(&conn, workspace_id.to_string(), access_token.to_string()) {
            Ok(is_authed) => {
                if !is_authed {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            },
            Err(_) => return Err(StatusCode::UNAUTHORIZED),
        }
    }

    req.extensions_mut().insert(workspace_id.to_string());
    Ok(next.run(req).await)
}

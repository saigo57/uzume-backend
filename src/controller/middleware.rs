use axum::response::Response;
use axum::{
    http::StatusCode,
    extract::{Request, State},
    middleware::Next,
};
use rusqlite::{Connection, params};
use std::sync::Arc;
use tokio::sync::Mutex;

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

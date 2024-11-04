use std::sync::Arc;
use rusqlite::Connection;
use tokio::sync::Mutex;
use crate::schema::create_schema;
use crate::model::file::writer::MockWriter;

#[allow(dead_code)] // テスト用コード
pub struct TestUtil {
    pub conn: Arc<Mutex<Connection>>,
    pub writer: MockWriter,
    pub workspace_id: String,
    pub workspace_path: String,
    pub test_access_token: String,
}

#[allow(dead_code)] // テスト用コード
impl TestUtil {
    pub async fn new() -> Self {
        let writer = MockWriter{data: Arc::new(std::sync::Mutex::new(None))};
        let conn = Connection::open_in_memory().unwrap();
        let conn = Arc::new(Mutex::new(conn));
        create_schema(conn.clone()).await.unwrap();

        let test_access_token = "test-access-token";
        let workspace_id = "12345678-xxxx-yyyy-zzzz-000000000000";
        let workspace_path = "/path/to/test.uzume";

        {
            let conn = conn.lock().await;
            conn.execute(
                "INSERT INTO auth (access_token, workspace_id) VALUES (?1, ?2)",
                [test_access_token, workspace_id],
            ).unwrap();

            conn.execute(
                "INSERT INTO config (path, workspace_id, name) VALUES (?1, ?2, ?3)",
                [workspace_path, workspace_id, "test_workspace"],
            ).unwrap();
        }

        TestUtil {
            conn,
            writer,
            workspace_id: workspace_id.to_string(),
            workspace_path: workspace_path.to_string(),
            test_access_token: test_access_token.to_string(),
        }
    }
}

use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkspaceInfo {
    #[schema(example = r"C:\Users\user\workspace")]
    pub path: String,

    #[schema(example = "a0b257bb-b7c6-46f3-b27c-31f8ce1c3703")]
    pub workspace_id: String,

    #[schema(example = "ワークスペース名")]
    pub name: String,
}

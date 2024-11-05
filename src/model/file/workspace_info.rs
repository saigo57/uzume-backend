use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = FileWorkspaceInfo)]
pub struct WorkspaceInfo {
    pub path: String,

    pub workspace_id: String,

    pub name: String,
}

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: String,

    pub workspace_id: String,

    pub name: String,
}

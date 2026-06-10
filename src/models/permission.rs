use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub id: i64,
    pub path: String,
    pub role: String,
    pub allowed_actions: Vec<ActionType>,
    pub inherit: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Read,
    Write,
    Delete,
    Upload,
    Download,
}

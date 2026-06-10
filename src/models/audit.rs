use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub operator: String,
    pub action_type: AuditActionType,
    pub target_path: Option<String>,
    pub detail: Option<String>,
    pub source_ip: String,
    pub result: AuditResult,
    pub failure_reason: Option<String>,
    pub action_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditActionType {
    Login,
    Logout,
    Upload,
    Download,
    Delete,
    Rename,
    Move,
    Copy,
    Mkdir,
    PermissionChange,
    UserManage,
}

impl AuditActionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "login" => Some(AuditActionType::Login),
            "logout" => Some(AuditActionType::Logout),
            "upload" => Some(AuditActionType::Upload),
            "download" => Some(AuditActionType::Download),
            "delete" => Some(AuditActionType::Delete),
            "rename" => Some(AuditActionType::Rename),
            "move" => Some(AuditActionType::Move),
            "copy" => Some(AuditActionType::Copy),
            "mkdir" => Some(AuditActionType::Mkdir),
            "permission_change" => Some(AuditActionType::PermissionChange),
            "user_manage" => Some(AuditActionType::UserManage),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
}

impl AuditResult {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "success" => Some(AuditResult::Success),
            "failure" => Some(AuditResult::Failure),
            _ => None,
        }
    }
}

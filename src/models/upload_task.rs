use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadTask {
    pub id: String,
    pub file_name: String,
    pub target_path: String,
    pub total_size: i64,
    pub chunk_size: i64,
    pub total_chunks: i32,
    pub uploaded_chunks: Vec<i32>,
    pub status: UploadStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UploadStatus {
    Pending,
    Uploading,
    Merging,
    Completed,
    Failed,
    Cancelled,
}

impl UploadStatus {
    pub fn as_str(&self) -> &str {
        match self {
            UploadStatus::Pending => "pending",
            UploadStatus::Uploading => "uploading",
            UploadStatus::Merging => "merging",
            UploadStatus::Completed => "completed",
            UploadStatus::Failed => "failed",
            UploadStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(UploadStatus::Pending),
            "uploading" => Some(UploadStatus::Uploading),
            "merging" => Some(UploadStatus::Merging),
            "completed" => Some(UploadStatus::Completed),
            "failed" => Some(UploadStatus::Failed),
            "cancelled" => Some(UploadStatus::Cancelled),
            _ => None,
        }
    }
}

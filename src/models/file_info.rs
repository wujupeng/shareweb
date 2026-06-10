use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub mime_type: Option<String>,
    pub preview_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryTreeNode {
    pub path: String,
    pub name: String,
    pub has_children: bool,
    pub expanded: bool,
    pub children: Vec<DirectoryTreeNode>,
}

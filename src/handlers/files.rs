use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::error::AppError;
use crate::services::file_service::FileService;

#[derive(Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
    pub show_hidden: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct TreeQuery {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub keyword: String,
    pub path: Option<String>,
    pub max_depth: Option<u32>,
}

#[derive(Deserialize)]
pub struct MkdirRequest {
    pub parent_path: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct RenameRequest {
    pub path: String,
    pub new_name: String,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub path: String,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub source_path: String,
    pub target_dir: String,
}

#[derive(Deserialize)]
pub struct CopyRequest {
    pub source_path: String,
    pub target_dir: String,
}

pub async fn list_files(
    file_service: web::Data<FileService>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let path = query.path.as_deref().unwrap_or("/");
    let show_hidden = query.show_hidden.unwrap_or(false);
    let sort_by = query.sort_by.as_deref().unwrap_or("name");
    let sort_order = query.sort_order.as_deref().unwrap_or("asc");
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50);

    let all_files = file_service.list_directory(path, show_hidden, sort_by, sort_order).await?;

    let total = all_files.len();
    let start = ((page - 1) * page_size) as usize;
    let end = std::cmp::min(start + page_size as usize, total);
    let items: Vec<_> = if start < total {
        all_files[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": {
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size
        }
    })))
}

pub async fn get_tree(
    file_service: web::Data<FileService>,
    query: web::Query<TreeQuery>,
) -> Result<HttpResponse, AppError> {
    let path = query.path.as_deref().unwrap_or("/");
    let tree = file_service.get_tree(path).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": tree
    })))
}

pub async fn search_files(
    file_service: web::Data<FileService>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let path = query.path.as_deref().unwrap_or("/");
    let max_depth = query.max_depth.unwrap_or(10);
    let results = file_service.search(&query.keyword, path, max_depth).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": results
    })))
}

pub async fn mkdir(
    file_service: web::Data<FileService>,
    body: web::Json<MkdirRequest>,
) -> Result<HttpResponse, AppError> {
    file_service.mkdir(&body.parent_path, &body.name).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "文件夹创建成功"
    })))
}

pub async fn rename(
    file_service: web::Data<FileService>,
    body: web::Json<RenameRequest>,
) -> Result<HttpResponse, AppError> {
    file_service.rename(&body.path, &body.new_name).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "重命名成功"
    })))
}

pub async fn delete(
    file_service: web::Data<FileService>,
    body: web::Json<DeleteRequest>,
) -> Result<HttpResponse, AppError> {
    file_service.delete(&body.path).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "删除成功"
    })))
}

pub async fn move_file(
    file_service: web::Data<FileService>,
    body: web::Json<MoveRequest>,
) -> Result<HttpResponse, AppError> {
    file_service.move_file(&body.source_path, &body.target_dir).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "移动成功"
    })))
}

pub async fn copy_file(
    file_service: web::Data<FileService>,
    body: web::Json<CopyRequest>,
) -> Result<HttpResponse, AppError> {
    file_service.copy_file(&body.source_path, &body.target_dir).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "复制成功"
    })))
}

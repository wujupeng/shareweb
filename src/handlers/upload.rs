use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::path::Path;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use uuid::Uuid;
use crate::error::AppError;
use crate::services::file_service::FileService;
use crate::utils::path_sanitizer::sanitize_path;
use crate::config::AppConfig;
use crate::middleware::auth::extract_token_from_header;
use crate::services::auth_service::AuthService;
use crate::repositories::audit_repo::AuditRepo;

#[derive(Deserialize)]
pub struct InitUploadRequest {
    pub file_name: String,
    pub target_path: String,
    pub total_size: i64,
}

pub async fn init_upload(
    config: web::Data<AppConfig>,
    body: web::Json<InitUploadRequest>,
) -> Result<HttpResponse, AppError> {
    if body.total_size > config.storage.max_file_size {
        return Err(AppError::FileTooLarge("文件超过大小限制(10GB)".to_string()));
    }

    let task_id = Uuid::new_v4().to_string();
    let chunk_size = config.storage.chunk_size;
    let total_chunks = ((body.total_size as f64) / chunk_size as f64).ceil() as i32;

    let tmp_dir = Path::new(&config.storage.tmp_dir).join(&task_id);
    tokio::fs::create_dir_all(&tmp_dir).await
        .map_err(|e| AppError::Internal(format!("创建临时目录失败: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": {
            "task_id": task_id,
            "chunk_size": chunk_size,
            "total_chunks": total_chunks
        }
    })))
}

#[derive(Deserialize)]
pub struct ChunkUploadQuery {
    pub task_id: String,
    pub chunk_index: i32,
}

pub async fn upload_chunk(
    config: web::Data<AppConfig>,
    query: web::Query<ChunkUploadQuery>,
    body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    let chunk_path = Path::new(&config.storage.tmp_dir)
        .join(&query.task_id)
        .join(format!("{}", query.chunk_index));

    tokio::fs::write(&chunk_path, &body).await
        .map_err(|e| AppError::Internal(format!("写入分片失败: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "分片上传成功",
        "data": { "chunk_index": query.chunk_index }
    })))
}

#[derive(Deserialize)]
pub struct CompleteUploadRequest {
    pub task_id: String,
    pub file_name: String,
    pub target_path: String,
    pub total_chunks: i32,
}

pub async fn complete_upload(
    config: web::Data<AppConfig>,
    file_service: web::Data<FileService>,
    auth_service: web::Data<AuthService>,
    db: web::Data<Arc<Mutex<Connection>>>,
    req: HttpRequest,
    body: web::Json<CompleteUploadRequest>,
) -> Result<HttpResponse, AppError> {
    let token = extract_token_from_header(req.headers())?;
    let claims = auth_service.verify_token(&token)?;
    let username = claims.sub;
    let source_ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();

    let safe_target = sanitize_path(&body.target_path, &file_service.base_dir)
        .map_err(|e| AppError::PathTraversal(e))?;
    let target_file = Path::new(&safe_target).join(&body.file_name);

    if target_file.exists() {
        return Err(AppError::Conflict("文件已存在".to_string()));
    }

    let tmp_dir = Path::new(&config.storage.tmp_dir).join(&body.task_id);
    let mut output = tokio::fs::File::create(&target_file).await
        .map_err(|e| AppError::Internal(format!("创建目标文件失败: {}", e)))?;

    use tokio::io::AsyncWriteExt;
    for i in 0..body.total_chunks {
        let chunk_path = tmp_dir.join(format!("{}", i));
        let data = tokio::fs::read(&chunk_path).await
            .map_err(|e| AppError::Internal(format!("读取分片{}失败: {}", i, e)))?;
        output.write_all(&data).await
            .map_err(|e| AppError::Internal(format!("合并分片{}失败: {}", i, e)))?;
    }
    output.flush().await
        .map_err(|e| AppError::Internal(format!("刷新文件失败: {}", e)))?;

    tokio::fs::remove_dir_all(&tmp_dir).await
        .map_err(|e| AppError::Internal(format!("清理临时文件失败: {}", e)))?;

    if let Ok(conn) = db.lock() {
        let _ = AuditRepo::insert(&conn, &username, "upload", Some(&body.target_path), Some(&body.file_name), &source_ip, "success", None);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "上传完成"
    })))
}

#[derive(Deserialize)]
pub struct UploadStatusQuery {
    pub task_id: String,
}

pub async fn upload_status(
    config: web::Data<AppConfig>,
    query: web::Query<UploadStatusQuery>,
) -> Result<HttpResponse, AppError> {
    let tmp_dir = Path::new(&config.storage.tmp_dir).join(&query.task_id);
    if !tmp_dir.exists() {
        return Err(AppError::NotFound("上传任务不存在".to_string()));
    }

    let mut uploaded: Vec<i32> = Vec::new();
    let mut entries = tokio::fs::read_dir(&tmp_dir).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| AppError::Internal(e.to_string()))? {
        if let Some(name) = entry.file_name().to_str() {
            if let Ok(idx) = name.parse::<i32>() {
                uploaded.push(idx);
            }
        }
    }
    uploaded.sort();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": {
            "task_id": query.task_id,
            "uploaded_chunks": uploaded
        }
    })))
}
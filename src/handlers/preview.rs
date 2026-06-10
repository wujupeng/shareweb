use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::path::Path;
use crate::error::AppError;
use crate::services::file_service::FileService;
use crate::utils::path_sanitizer::sanitize_path;
use crate::utils::file_type::get_preview_type;

#[derive(Deserialize)]
pub struct PreviewQuery {
    pub path: String,
}

pub async fn preview_file(
    file_service: web::Data<FileService>,
    query: web::Query<PreviewQuery>,
) -> Result<HttpResponse, AppError> {
    let safe_path = sanitize_path(&query.path, &file_service.base_dir)
        .map_err(|e| AppError::PathTraversal(e))?;
    let full_path = Path::new(&safe_path);

    if !full_path.exists() {
        return Err(AppError::NotFound("文件不存在".to_string()));
    }

    let metadata = tokio::fs::metadata(&full_path).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if metadata.len() > 50 * 1024 * 1024 {
        return Err(AppError::FileTooLarge("文件超过50MB，不支持在线预览，请下载查看".to_string()));
    }

    let extension = full_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let preview_type = get_preview_type(extension);

    let file_name = full_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    match preview_type {
        crate::utils::file_type::PreviewType::Image => {
            let data = tokio::fs::read(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mime = mime_guess::from_ext(extension).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", mime.to_string()))
                .body(data))
        }
        crate::utils::file_type::PreviewType::Text => {
            let data = tokio::fs::read(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let content = String::from_utf8_lossy(&data).to_string();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "code": 0,
                "message": "success",
                "data": {
                    "content": content,
                    "encoding": "utf-8",
                    "mime_type": "text/plain",
                    "size": metadata.len(),
                    "filename": file_name
                }
            })))
        }
        crate::utils::file_type::PreviewType::Pdf => {
            let data = tokio::fs::read(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", "application/pdf"))
                .body(data))
        }
        crate::utils::file_type::PreviewType::Video => {
            let data = tokio::fs::read(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mime = mime_guess::from_ext(extension).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", mime.to_string()))
                .insert_header(("Accept-Ranges", "bytes"))
                .body(data))
        }
        crate::utils::file_type::PreviewType::Audio => {
            let data = tokio::fs::read(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mime = mime_guess::from_ext(extension).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", mime.to_string()))
                .body(data))
        }
        crate::utils::file_type::PreviewType::None => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "code": 2001,
                "message": "不支持预览该文件类型",
                "data": {
                    "download_url": format!("/api/files/download?path={}", urlencoding::encode(&query.path))
                }
            })))
        }
    }
}

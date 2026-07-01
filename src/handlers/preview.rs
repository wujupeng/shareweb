use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::path::Path;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use crate::error::AppError;
use crate::services::file_service::FileService;
use crate::utils::path_sanitizer::sanitize_path;
use crate::utils::file_type::get_preview_type;
use crate::middleware::auth::extract_token_from_header;
use crate::services::auth_service::AuthService;
use crate::repositories::permission_repo::PermissionRepo;
use crate::repositories::audit_repo::AuditRepo;

#[derive(Deserialize)]
pub struct PreviewQuery {
    pub path: String,
    pub token: Option<String>,
}

fn get_user_role(req: &HttpRequest, auth_service: &AuthService, query_token: Option<&str>) -> Result<String, AppError> {
    let token = query_token
        .map(|t| t.to_string())
        .or_else(|| extract_token_from_header(req.headers()).ok());
    let token = token.ok_or_else(|| AppError::Unauthorized("缺少认证头".to_string()))?;
    let claims = auth_service.verify_token(&token)?;
    Ok(claims.role)
}

fn get_user_username(req: &HttpRequest, auth_service: &AuthService, query_token: Option<&str>) -> Result<String, AppError> {
    let token = query_token
        .map(|t| t.to_string())
        .or_else(|| extract_token_from_header(req.headers()).ok());
    let token = token.ok_or_else(|| AppError::Unauthorized("缺少认证头".to_string()))?;
    let claims = auth_service.verify_token(&token)?;
    Ok(claims.sub)
}

fn get_allowed_paths(db: &Arc<Mutex<Connection>>, role: &str) -> Vec<String> {
    if role == "admin" {
        return vec!["/".to_string()];
    }
    let conn = db.lock().unwrap();
    let all_rules = PermissionRepo::list(&conn, None).unwrap_or_default();
    all_rules.iter()
        .filter(|r| r.role == role)
        .map(|r| r.path.clone())
        .collect()
}

fn is_path_allowed(requested_path: &str, allowed_paths: &[String]) -> bool {
    if allowed_paths.contains(&"/".to_string()) {
        return true;
    }
    for allowed in allowed_paths {
        let norm_allowed = allowed.trim_end_matches('/');
        let norm_requested = requested_path.trim_end_matches('/');
        if norm_requested == norm_allowed
            || norm_requested.starts_with(&format!("{}/", norm_allowed))
            || norm_allowed.starts_with(&format!("{}/", norm_requested))
        {
            return true;
        }
    }
    false
}

pub async fn preview_file(
    file_service: web::Data<FileService>,
    auth_service: web::Data<AuthService>,
    db: web::Data<Arc<Mutex<Connection>>>,
    query: web::Query<PreviewQuery>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let role = get_user_role(&req, &auth_service, query.token.as_deref())?;
    let username = get_user_username(&req, &auth_service, query.token.as_deref())?;
    let source_ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
    let allowed_paths = get_allowed_paths(&db, &role);
    if !is_path_allowed(&query.path, &allowed_paths) {
        if let Ok(conn) = db.lock() {
            let _ = AuditRepo::insert(&conn, &username, "preview", Some(&query.path), None, &source_ip, "failure", Some("无权访问此路径"));
        }
        return Err(AppError::Forbidden("无权访问此路径".to_string()));
    }

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

    if let Ok(conn) = db.lock() {
        let _ = AuditRepo::insert(&conn, &username, "preview", Some(&query.path), None, &source_ip, "success", None);
    }

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
            let chunk_size = 8 * 1024 * 1024;
            let file = tokio::fs::File::open(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mime = mime_guess::from_ext(extension).first_or_octet_stream();
            use actix_web::body::BodyStream;
            use futures::stream::StreamExt;
            let stream = async_stream::stream! {
                let mut buf = vec![0u8; chunk_size];
                use tokio::io::AsyncReadExt;
                let mut file = file;
                loop {
                    match file.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            yield Ok::<_, std::io::Error>(bytes::Bytes::copy_from_slice(&buf[..n]));
                        }
                        Err(e) => {
                            yield Err(e);
                            break;
                        }
                    }
                }
            };
            Ok(HttpResponse::Ok()
                .insert_header(("Content-Type", mime.to_string()))
                .insert_header(("Content-Length", metadata.len()))
                .insert_header(("Accept-Ranges", "bytes"))
                .body(BodyStream::new(stream.boxed())))
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
            let download_url = if let Some(ref t) = query.token {
                format!("/api/files/download?path={}&token={}", urlencoding::encode(&query.path), urlencoding::encode(t))
            } else {
                format!("/api/files/download?path={}", urlencoding::encode(&query.path))
            };
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "code": 2001,
                "message": "不支持预览该文件类型",
                "data": {
                    "download_url": download_url
                }
            })))
        }
    }
}

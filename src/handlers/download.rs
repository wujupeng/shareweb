use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::path::Path;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use crate::error::AppError;
use crate::services::file_service::FileService;
use crate::utils::path_sanitizer::sanitize_path;
use crate::middleware::auth::extract_token_from_header;
use crate::services::auth_service::AuthService;
use crate::repositories::permission_repo::PermissionRepo;
use crate::repositories::audit_repo::AuditRepo;

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub path: String,
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct BatchDownloadRequest {
    pub paths: Vec<String>,
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

pub async fn download_file(
    file_service: web::Data<FileService>,
    auth_service: web::Data<AuthService>,
    db: web::Data<Arc<Mutex<Connection>>>,
    query: web::Query<DownloadQuery>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let role = get_user_role(&req, &auth_service, query.token.as_deref())?;
    let username = get_user_username(&req, &auth_service, query.token.as_deref())?;
    let source_ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
    let allowed_paths = get_allowed_paths(&db, &role);
    if !is_path_allowed(&query.path, &allowed_paths) {
        if let Ok(conn) = db.lock() {
            let _ = AuditRepo::insert(&conn, &username, "download", Some(&query.path), None, &source_ip, "failure", Some("无权访问此路径"));
        }
        return Err(AppError::Forbidden("无权访问此路径".to_string()));
    }

    let safe_path = sanitize_path(&query.path, &file_service.base_dir)
        .map_err(|e| AppError::PathTraversal(e))?;
    let full_path = Path::new(&safe_path);

    if let Ok(conn) = db.lock() {
        let _ = AuditRepo::insert(&conn, &username, "download", Some(&query.path), None, &source_ip, "success", None);
    }

    if !full_path.exists() {
        return Err(AppError::NotFound("文件不存在".to_string()));
    }
    if full_path.is_dir() {
        return Err(AppError::BadRequest("不能直接下载目录，请使用批量下载".to_string()));
    }

    let file_name = full_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    let file_size = tokio::fs::metadata(&full_path).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .len();

    if let Some(range_header) = req.headers().get("Range") {
        if let Ok(range_str) = range_header.to_str() {
            if let Some(range) = parse_range(range_str, file_size) {
                let chunk_size = (range.end - range.start + 1).min(8 * 1024 * 1024) as usize;
                let file = tokio::fs::File::open(&full_path).await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                use tokio::io::AsyncSeekExt;
                let mut file = file;
                file.seek(std::io::SeekFrom::Start(range.start)).await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                use actix_web::body::BodyStream;
                use futures::stream::StreamExt;
                let remaining = range.end - range.start + 1;
                let stream = async_stream::stream! {
                    let mut pos: u64 = 0;
                    let mut buf = vec![0u8; chunk_size];
                    use tokio::io::AsyncReadExt;
                    while pos < remaining {
                        let to_read = std::cmp::min(chunk_size as u64, remaining - pos) as usize;
                        match file.read(&mut buf[..to_read]).await {
                            Ok(0) => break,
                            Ok(n) => {
                                pos += n as u64;
                                yield Ok::<_, std::io::Error>(bytes::Bytes::copy_from_slice(&buf[..n]));
                            }
                            Err(e) => {
                                yield Err(e);
                                break;
                            }
                        }
                    }
                };
                return Ok(HttpResponse::PartialContent()
                    .insert_header(("Content-Range", format!("bytes {}-{}/{}", range.start, range.end, file_size)))
                    .insert_header(("Content-Length", range.end - range.start + 1))
                    .insert_header(("Content-Type", "application/octet-stream"))
                    .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", urlencoding::encode(&file_name))))
                    .body(BodyStream::new(stream.boxed())));
            }
        }
    }

    let chunk_size = 8 * 1024 * 1024;
    let file = tokio::fs::File::open(&full_path).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
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
        .insert_header(("Content-Type", "application/octet-stream"))
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", urlencoding::encode(&file_name))))
        .insert_header(("Content-Length", file_size))
        .insert_header(("Accept-Ranges", "bytes"))
        .body(BodyStream::new(stream.boxed())))
}

pub async fn batch_download(
    file_service: web::Data<FileService>,
    auth_service: web::Data<AuthService>,
    db: web::Data<Arc<Mutex<Connection>>>,
    req: HttpRequest,
    body: web::Json<BatchDownloadRequest>,
) -> Result<HttpResponse, AppError> {
    let role = get_user_role(&req, &auth_service, None)?;
    let username = get_user_username(&req, &auth_service, None)?;
    let source_ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
    let allowed_paths = get_allowed_paths(&db, &role);

    for p in &body.paths {
        if !is_path_allowed(p, &allowed_paths) {
            if let Ok(conn) = db.lock() {
                let _ = AuditRepo::insert(&conn, &username, "batch_download", Some(p), None, &source_ip, "failure", Some("无权访问此路径"));
            }
            return Err(AppError::Forbidden("无权访问此路径".to_string()));
        }
    }

    if let Ok(conn) = db.lock() {
        let paths_str = body.paths.join(",");
        let _ = AuditRepo::insert(&conn, &username, "batch_download", Some(&paths_str), None, &source_ip, "success", None);
    }

    use std::io::Write;
    let mut buffer = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for path in &body.paths {
            let safe_path = sanitize_path(path, &file_service.base_dir)
                .map_err(|e| AppError::PathTraversal(e))?;
            let full_path = Path::new(&safe_path);
            if !full_path.exists() || full_path.is_dir() { continue; }

            let data = tokio::fs::read(&full_path).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let entry_name = full_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            zip.start_file(entry_name, options)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            zip.write_all(&data)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        zip.finish().map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "application/zip"))
        .insert_header(("Content-Disposition", "attachment; filename=\"download.zip\""))
        .insert_header(("Content-Length", buffer.len()))
        .body(buffer))
}

struct Range { start: u64, end: u64 }

fn parse_range(range_str: &str, file_size: u64) -> Option<Range> {
    if !range_str.starts_with("bytes=") { return None; }
    let range_spec = &range_str[6..];
    let parts: Vec<&str> = range_spec.split('-').collect();
    if parts.len() != 2 { return None; }
    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() { file_size - 1 } else { parts[1].parse().ok()? };
    Some(Range { start, end: end.min(file_size - 1) })
}
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::path::Path;
use crate::error::AppError;
use crate::services::file_service::FileService;
use crate::utils::path_sanitizer::sanitize_path;

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct BatchDownloadRequest {
    pub paths: Vec<String>,
}

pub async fn download_file(
    file_service: web::Data<FileService>,
    query: web::Query<DownloadQuery>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let safe_path = sanitize_path(&query.path, &file_service.base_dir)
        .map_err(|e| AppError::PathTraversal(e))?;
    let full_path = Path::new(&safe_path);

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
                let data = read_range(&full_path, range.start, range.end).await?;
                return Ok(HttpResponse::PartialContent()
                    .insert_header(("Content-Range", format!("bytes {}-{}/{}", range.start, range.end, file_size)))
                    .insert_header(("Content-Length", range.end - range.start + 1))
                    .insert_header(("Content-Type", "application/octet-stream"))
                    .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", urlencoding::encode(&file_name))))
                    .body(data));
            }
        }
    }

    let data = tokio::fs::read(&full_path).await
        .map_err(|e| AppError::Internal(format!("读取文件失败: {}", e)))?;

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "application/octet-stream"))
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", urlencoding::encode(&file_name))))
        .insert_header(("Content-Length", file_size))
        .insert_header(("Accept-Ranges", "bytes"))
        .body(data))
}

pub async fn batch_download(
    file_service: web::Data<FileService>,
    body: web::Json<BatchDownloadRequest>,
) -> Result<HttpResponse, AppError> {
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

async fn read_range(path: &Path, start: u64, end: u64) -> Result<Vec<u8>, AppError> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    let mut file = tokio::fs::File::open(path).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    file.seek(std::io::SeekFrom::Start(start)).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let len = (end - start + 1) as usize;
    let mut buffer = vec![0u8; len];
    file.read_exact(&mut buffer).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(buffer)
}

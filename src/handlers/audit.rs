use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use crate::error::AppError;
use crate::repositories::audit_repo::AuditRepo;

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub operator: Option<String>,
    pub action_type: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn list_audit_logs(
    db: web::Data<Arc<Mutex<Connection>>>,
    query: web::Query<AuditLogQuery>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50).min(100);
    let offset = ((page - 1) * page_size) as i64;

    let logs = AuditRepo::list(
        &conn,
        query.operator.as_deref(),
        query.action_type.as_deref(),
        query.start_time.as_deref(),
        query.end_time.as_deref(),
        page_size as i64,
        offset,
    ).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": { "items": logs, "page": page, "page_size": page_size }
    })))
}

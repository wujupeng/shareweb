use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use crate::error::AppError;
use crate::repositories::permission_repo::PermissionRepo;

#[derive(Deserialize)]
pub struct ListPermissionsQuery {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePermissionRequest {
    pub path: String,
    pub role: String,
    pub allowed_actions: Vec<String>,
    pub inherit: Option<bool>,
}

pub async fn list_permissions(
    db: web::Data<Arc<Mutex<Connection>>>,
    query: web::Query<ListPermissionsQuery>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let rules = PermissionRepo::list(&conn, query.path.as_deref())
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": rules
    })))
}

pub async fn create_permission(
    db: web::Data<Arc<Mutex<Connection>>>,
    body: web::Json<CreatePermissionRequest>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let actions_json = serde_json::to_string(&body.allowed_actions).unwrap_or_default();
    let inherit = body.inherit.unwrap_or(true);
    PermissionRepo::create(&conn, &body.path, &body.role, &actions_json, inherit)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "code": 0, "message": "权限规则创建成功" })))
}

pub async fn update_permission() -> Result<HttpResponse, AppError> {
    todo!()
}

pub async fn delete_permission(
    db: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    PermissionRepo::delete(&conn, *path).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "code": 0, "message": "权限规则删除成功" })))
}

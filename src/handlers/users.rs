use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use crate::error::AppError;
use crate::repositories::user_repo::UserRepo;
use crate::config::AppConfig;

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<String>,
    pub status: Option<String>,
    pub password: Option<String>,
}

pub async fn list_users(
    db: web::Data<Arc<Mutex<Connection>>>,
    query: web::Query<ListUsersQuery>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = ((page - 1) * page_size) as i64;

    let users = UserRepo::list(&conn, query.status.as_deref(), page_size as i64, offset)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut safe_users: Vec<serde_json::Value> = Vec::new();
    for u in &users {
        safe_users.push(serde_json::json!({
            "username": u.username,
            "role": u.role.as_str(),
            "status": u.status.as_str(),
            "created_at": u.created_at,
            "updated_at": u.updated_at,
        }));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": { "items": safe_users, "page": page, "page_size": page_size }
    })))
}

pub async fn create_user(
    db: web::Data<Arc<Mutex<Connection>>>,
    config: web::Data<AppConfig>,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    if UserRepo::find_by_username(&conn, &body.username).map_err(|e| AppError::Internal(e.to_string()))?.is_some() {
        return Err(AppError::Conflict("用户名已存在".to_string()));
    }
    let hash = bcrypt::hash(&body.password, config.auth.bcrypt_cost)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let role = body.role.as_deref().unwrap_or("readonly");
    UserRepo::create(&conn, &body.username, &hash, role)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "code": 0, "message": "用户创建成功" })))
}

pub async fn update_user(
    db: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<String>,
    body: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let username = path.into_inner();
    let role = body.role.as_deref().unwrap_or("readonly");
    let status = body.status.as_deref().unwrap_or("active");
    UserRepo::update_role_and_status(&conn, &username, role, status)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(ref pwd) = body.password {
        let hash = bcrypt::hash(pwd, 12).map_err(|e| AppError::Internal(e.to_string()))?;
        UserRepo::update_password(&conn, &username, &hash)
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({ "code": 0, "message": "用户更新成功" })))
}

pub async fn delete_user(
    db: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let username = path.into_inner();
    let admin_count = UserRepo::count_admins(&conn).map_err(|e| AppError::Internal(e.to_string()))?;
    let user = UserRepo::find_by_username(&conn, &username)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;
    if user.role.as_str() == "admin" && admin_count <= 1 {
        return Err(AppError::Forbidden("不能删除最后一个管理员".to_string()));
    }
    UserRepo::delete(&conn, &username).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "code": 0, "message": "用户删除成功" })))
}

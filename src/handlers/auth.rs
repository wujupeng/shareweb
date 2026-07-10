use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use serde::Deserialize;
use std::sync::{Mutex, Arc};
use rusqlite::Connection;
use crate::error::AppError;
use crate::services::auth_service::AuthService;
use crate::repositories::audit_repo::AuditRepo;
use crate::middleware::auth::extract_token_from_header;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn login(
    auth_service: web::Data<AuthService>,
    db: web::Data<Arc<Mutex<Connection>>>,
    req: HttpRequest,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let source_ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
    let result = auth_service.login(&body.username, &body.password);
    match result {
        Ok((token, role, expires_in)) => {
            if let Ok(conn) = db.lock() {
                let _ = AuditRepo::insert(&conn, &body.username, "login", None, None, &source_ip, "success", None);
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "code": 0,
                "message": "success",
                "data": {
                    "token": token,
                    "role": role,
                    "expires_in": expires_in
                }
            })))
        }
        Err(e) => {
            if let Ok(conn) = db.lock() {
                let _ = AuditRepo::insert(&conn, &body.username, "login", None, None, &source_ip, "failure", Some(&e.to_string()));
            }
            Err(e)
        }
    }
}

pub async fn logout() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "已登出"
    })))
}

pub async fn profile(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let token = extract_token_from_header(req.headers())?;
    let claims = auth_service.verify_token(&token)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": {
            "username": claims.sub,
            "role": claims.role
        }
    })))
}

pub async fn change_password(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let token = extract_token_from_header(req.headers())?;
    let claims = auth_service.verify_token(&token)?;
    auth_service.change_password(&claims.sub, &body.old_password, &body.new_password)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "密码修改成功"
    })))
}
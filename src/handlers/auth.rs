use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use serde::Deserialize;
use crate::error::AppError;
use crate::services::auth_service::AuthService;

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
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let (token, role, expires_in) = auth_service.login(&body.username, &body.password)?;
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

pub async fn logout() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "已登出"
    })))
}

pub async fn profile(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let username = req.extensions().get::<String>().cloned().unwrap_or_default();
    let role = req.extensions().get::<String>().cloned().unwrap_or_default();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "success",
        "data": {
            "username": username,
            "role": role
        }
    })))
}

pub async fn change_password(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let username = req.extensions().get::<String>().cloned()
        .ok_or_else(|| AppError::Unauthorized("未认证".to_string()))?;
    auth_service.change_password(&username, &body.old_password, &body.new_password)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 0,
        "message": "密码修改成功"
    })))
}

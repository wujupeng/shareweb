use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web::error::ErrorUnauthorized;
use actix_web::web::Data;
use crate::error::AppError;
use crate::services::auth_service::AuthService;

pub fn extract_token_from_header(headers: &actix_web::http::header::HeaderMap) -> Result<String, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("缺少认证头".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("认证头格式错误".to_string()));
    }

    Ok(auth_header[7..].to_string())
}

pub struct AuthInfo {
    pub username: String,
    pub role: String,
}

pub fn verify_request(req: &ServiceRequest, auth_service: &AuthService) -> Result<AuthInfo, Error> {
    let token = extract_token_from_header(req.headers())
        .map_err(|e| ErrorUnauthorized(serde_json::json!({"code": e.error_code(), "message": e.to_string()})))?;

    let claims = auth_service.verify_token(&token)
        .map_err(|e| ErrorUnauthorized(serde_json::json!({"code": e.error_code(), "message": e.to_string()})))?;

    Ok(AuthInfo {
        username: claims.sub,
        role: claims.role,
    })
}

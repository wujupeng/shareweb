use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("认证失败: {0}")]
    Unauthorized(String),
    #[error("令牌已过期: {0}")]
    TokenExpired(String),
    #[error("权限不足: {0}")]
    Forbidden(String),
    #[error("资源未找到: {0}")]
    NotFound(String),
    #[error("请求参数错误: {0}")]
    BadRequest(String),
    #[error("文件已存在: {0}")]
    Conflict(String),
    #[error("文件过大: {0}")]
    FileTooLarge(String),
    #[error("路径遍历攻击: {0}")]
    PathTraversal(String),
    #[error("内部服务器错误: {0}")]
    Internal(String),
}

impl AppError {
    pub fn error_code(&self) -> i32 {
        match self {
            AppError::Unauthorized(_) => 1001,
            AppError::TokenExpired(_) => 1002,
            AppError::Forbidden(_) => 1003,
            AppError::NotFound(_) => 2001,
            AppError::BadRequest(_) => 2002,
            AppError::Conflict(_) => 2003,
            AppError::FileTooLarge(_) => 3001,
            AppError::PathTraversal(_) => 3002,
            AppError::Internal(_) => 4001,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        })
    }

    pub fn success_message(message: &str) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse::<()> {
            code: 0,
            message: message.to_string(),
            data: None,
        })
    }

    pub fn error(status: StatusCode, code: i32, message: &str) -> HttpResponse {
        HttpResponse::build(status).json(ApiResponse::<()> {
            code,
            message: message.to_string(),
            data: None,
        })
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized(_) | AppError::TokenExpired(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::PathTraversal(_) => StatusCode::FORBIDDEN,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        ApiResponse::<()>::error(
            self.status_code(),
            self.error_code(),
            &self.to_string(),
        )
    }
}

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod utils;

use std::sync::{Mutex, Arc};

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use rusqlite::Connection;
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::AppConfig;
use crate::handlers::{auth, health, files, upload, download, preview, users, permissions, audit};
use crate::repositories::user_repo::UserRepo;
use crate::services::auth_service::AuthService;
use crate::services::file_service::FileService;

fn init_database(config: &AppConfig) -> Result<Arc<Mutex<Connection>>, String> {
    if let Some(parent) = std::path::Path::new(&config.database.path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败: {}", e))?;
    }

    let conn = Connection::open(&config.database.path)
        .map_err(|e| format!("打开数据库失败: {}", e))?;

    let migration_sql = include_str!("../migrations/001_init.sql");
    conn.execute_batch(migration_sql)
        .map_err(|e| format!("执行迁移失败: {}", e))?;

    let admin_hash = bcrypt::hash("Admin@2026", config.auth.bcrypt_cost)
        .map_err(|e| format!("管理员密码哈希失败: {}", e))?;
    UserRepo::init_admin(&conn, &admin_hash)
        .map_err(|e| format!("初始化管理员失败: {}", e))?;

    tracing::info!("数据库初始化完成: {}", config.database.path);
    Ok(Arc::new(Mutex::new(conn)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    let config = AppConfig::load(&config_path).unwrap_or_else(|e| {
        eprintln!("配置加载失败: {}", e);
        std::process::exit(1);
    });

    if let Err(e) = config.validate() {
        eprintln!("配置校验失败: {}", e);
        std::process::exit(1);
    }

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.log.level)),
        )
        .init();

    tracing::info!("启动 Web 文件管理系统");
    tracing::info!("监听 {}:{}", config.server.host, config.server.port);
    tracing::info!("共享目录: {}", config.storage.base_dir);

    let db = init_database(&config).unwrap_or_else(|e| {
        eprintln!("数据库初始化失败: {}", e);
        std::process::exit(1);
    });

    let auth_service = AuthService::new(db.clone(), config.clone());
    let file_service = FileService::new(&config.storage.base_dir);

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let config_data = web::Data::new(config.clone());
    let auth_data = web::Data::new(auth_service);
    let file_data = web::Data::new(file_service);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(actix_middleware::Logger::default())
            .app_data(web::PayloadConfig::new(100 * 1024 * 1024).limit(100 * 1024 * 1024))
            .app_data(config_data.clone())
            .app_data(auth_data.clone())
            .app_data(file_data.clone())
            .app_data(web::Data::new(db.clone()))
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health::health_check))
                    .route("/auth/login", web::post().to(auth::login))
                    .route("/auth/logout", web::post().to(auth::logout))
                    .route("/auth/profile", web::get().to(auth::profile))
                    .route("/auth/password", web::put().to(auth::change_password))
                    .route("/files", web::get().to(files::list_files))
                    .route("/files/tree", web::get().to(files::get_tree))
                    .route("/files/search", web::get().to(files::search_files))
                    .route("/files/mkdir", web::post().to(files::mkdir))
                    .route("/files/rename", web::put().to(files::rename))
                    .route("/files/delete", web::delete().to(files::delete))
                    .route("/files/move", web::post().to(files::move_file))
                    .route("/files/copy", web::post().to(files::copy_file))
                    .route("/files/download", web::get().to(download::download_file))
                    .route("/files/download/batch", web::post().to(download::batch_download))
                    .route("/files/preview", web::get().to(preview::preview_file))
                    .route("/upload/init", web::post().to(upload::init_upload))
                    .route("/upload/chunk", web::post().to(upload::upload_chunk))
                    .route("/upload/complete", web::post().to(upload::complete_upload))
                    .route("/upload/status", web::get().to(upload::upload_status))
                    .route("/users", web::get().to(users::list_users))
                    .route("/users", web::post().to(users::create_user))
                    .route("/users/{username}", web::put().to(users::update_user))
                    .route("/users/{username}", web::delete().to(users::delete_user))
                    .route("/permissions", web::get().to(permissions::list_permissions))
                    .route("/permissions", web::post().to(permissions::create_permission))
                    .route("/permissions/{id}", web::delete().to(permissions::delete_permission))
                    .route("/audit-logs", web::get().to(audit::list_audit_logs))
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

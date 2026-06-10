use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub base_dir: String,
    pub max_file_size: i64,
    pub chunk_size: i64,
    pub tmp_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub max_login_attempts: i32,
    pub lock_duration_minutes: i32,
    pub bcrypt_cost: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub file_path: Option<String>,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        if let Ok(secret) = env::var("JWT_SECRET") {
            config.auth.jwt_secret = secret;
        }
        if let Ok(base_dir) = env::var("STORAGE_BASE_DIR") {
            config.storage.base_dir = base_dir;
        }
        if let Ok(db_path) = env::var("DATABASE_PATH") {
            config.database.path = db_path;
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.auth.jwt_secret.is_empty() {
            return Err("jwt_secret 不能为空".to_string());
        }
        if self.storage.base_dir.is_empty() {
            return Err("storage.base_dir 不能为空".to_string());
        }
        if self.database.path.is_empty() {
            return Err("database.path 不能为空".to_string());
        }
        if self.auth.bcrypt_cost < 4 || self.auth.bcrypt_cost > 31 {
            return Err("bcrypt_cost 必须在 4-31 之间".to_string());
        }
        Ok(())
    }
}

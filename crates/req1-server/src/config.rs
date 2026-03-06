use std::env;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub listen_addr: String,
    pub cors_origin: Option<String>,
    pub static_dir: Option<String>,
    pub build_sha: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let listen_addr = if let Ok(port) = env::var("PORT") {
            format!("0.0.0.0:{port}")
        } else {
            env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        };

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").ok(),
            listen_addr,
            cors_origin: env::var("CORS_ORIGIN").ok(),
            static_dir: env::var("STATIC_DIR").ok(),
            build_sha: env::var("BUILD_SHA").ok(),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        }
    }
}

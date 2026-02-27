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
        }
    }
}

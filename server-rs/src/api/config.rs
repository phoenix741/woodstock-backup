//! Configuration for the public API server

use std::env;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    /// Hostname to bind to
    pub hostname: String,

    /// Port for the public API server (default: 3000)
    pub port: u16,

    /// Path to static files to serve (e.g. frontend)
    pub static_path: Option<String>,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            hostname: env::var("MANAGEMENT_API_LISTEN").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("MANAGEMENT_API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            static_path: env::var("STATIC_PATH").ok(),
        }
    }
}

impl ApiServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Get API bind address
    pub fn api_address(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }
}

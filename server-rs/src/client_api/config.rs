//! Configuration module for server-rs
//!
//! This module provides additional configuration for server components,
//! building upon the core woodstock-rs configuration.

use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for the client API server (mTLS authenticated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientApiConfig {
    /// Hostname to bind to
    pub hostname: String,

    /// Port to listen on  
    pub port: u16,
}

impl Default for ClientApiConfig {
    fn default() -> Self {
        Self {
            hostname: env::var("CLIENT_API_LISTEN").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("CLIENT_API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8443),
        }
    }
}

impl ClientApiConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Get client API bind address
    pub fn client_api_address(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }
}

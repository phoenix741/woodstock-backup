//! # Public API modules
//!
//! This module contains the implementation of the public-facing REST and GraphQL API
//! that replaces the NestJS api application. It serves the Vue.js frontend.

pub mod config;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod routes;
pub mod services;
pub mod state;

pub use config::ApiServerConfig;
pub use dto::*;
pub use error::{ApiError, ApiErrorResponse, ApiResult};
pub use state::ApiServerState;

//! Error handling for `server-rs`.
//!
//! The canonical error type for HTTP handlers is [`crate::api::ApiError`].
//! This module re-exports it at the crate root so callers can use
//! `woodstock_server_rs::ApiError` directly.

pub use crate::api::error::{ApiError, ApiErrorResponse, ApiResult};

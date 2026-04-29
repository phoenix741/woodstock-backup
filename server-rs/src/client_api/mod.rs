//! Client API module - mTLS authenticated endpoints for backup clients

pub mod auth;
pub mod config;
pub mod docs;
pub mod dto;
pub mod handlers;
pub mod middleware;
pub mod validation;

pub use auth::*;
pub use config::*;
pub use docs::*;
pub use dto::*;
pub use handlers::*;
pub use middleware::*;
pub use validation::*;

#![recursion_limit = "256"]

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod scanner;

#[cfg(feature = "pool")]
pub mod pool;

#[cfg(feature = "server")]
pub mod server;

pub mod config;
pub mod manifest;
pub mod proto;
pub mod statistics;
pub mod utils;

mod woodstock {
    tonic::include_proto!("woodstock");
}

pub use woodstock::*;

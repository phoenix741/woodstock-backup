#![recursion_limit = "256"]

pub mod authentification;
pub mod client;
pub mod commands;
pub mod config;
pub mod manifest;
pub mod pool;
pub mod proto;
pub mod scanner;
pub mod server;
pub mod utils;
pub mod statistics;

mod woodstock {
    tonic::include_proto!("woodstock");
}

pub use woodstock::*;

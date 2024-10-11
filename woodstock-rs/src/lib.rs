#![recursion_limit = "512"]

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod scanner;

#[cfg(feature = "pool")]
pub mod pool;

#[cfg(feature = "server")]
pub mod server;

pub mod config;
pub mod events;
pub mod manifest;
pub mod proto;
pub mod statistics;
pub mod utils;
pub mod view;

mod woodstock {
    #![allow(clippy::all, clippy::pedantic)]
    tonic::include_proto!("woodstock");
}

pub use woodstock::*;

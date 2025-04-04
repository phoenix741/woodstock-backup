#![recursion_limit = "512"]

pub mod client;
pub mod scanner;

pub mod pool;
pub mod view;

pub mod server;

pub mod config;
pub mod events;
pub mod manifest;
pub mod proto;
pub mod statistics;
pub mod utils;

mod woodstock {
    #![allow(clippy::all, clippy::pedantic)]
    tonic::include_proto!("woodstock");
}

pub use woodstock::*;

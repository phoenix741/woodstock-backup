#![deny(clippy::all)]

mod index_manifest_model;
mod manifest_model;
mod protobuf_service;

pub mod woodstock {
  include!(concat!(env!("OUT_DIR"), "/woodstock.rs"));
}

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

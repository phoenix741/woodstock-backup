//! # Build Script for Shared Rust Library
//!
//! This file is a build script for the shared library.
//! It configures the integration with Node.js via Node-API (N-API) using the `napi_build` crate.
//!
//! ## Functionality
//!
//! The script is automatically executed by Cargo before compiling the main project.
//! It generates the necessary bindings for Rust code to be called from JavaScript/Node.js.

/// Imports the `napi_build` crate which provides tools for generating Node-API bindings
extern crate napi_build;

/// Main build script function
///
/// This function is automatically called by Cargo during compilation.
/// It configures the build environment for Node-API bindings.
fn main() {
  napi_build::setup();
}

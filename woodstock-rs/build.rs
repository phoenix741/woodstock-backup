//! Build script for woodstock-rs
//!
//! This script has two main responsibilities:
//! 1. Configure and compile Protocol Buffers definitions with the `tonic-prost-build` crate
//! 2. Handle special build requirements for musl targets

use std::io::Result;

/// Main build function that runs before the main compilation process
///
/// This function configures the Protocol Buffer compiler to:
/// - Add serde support to all generated messages and enums
/// - Apply special serialization attributes to specific fields
/// - Compile the woodstock.proto file into Rust code
fn main() -> Result<()> {
    // Configure tonic-prost-build for protobuf compilation with serde support
    tonic_prost_build::configure()
        // Add serde serialization support to all messages
        .message_attribute(".", "#[serde_with::serde_as]\n#[derive(serde::Serialize)]")
        // Add serde serialization support to all enums
        .enum_attribute(".", "#[derive(serde::Serialize)]")
        // Configure special serialization for binary fields
        // SHA256 hashes are serialized as hexadecimal strings
        .field_attribute(
            "PoolUnused.sha256",
            "#[serde_as(as = \"serde_with::hex::Hex\")]",
        )
        .field_attribute(
            "PoolRefCount.sha256",
            "#[serde_as(as = \"serde_with::hex::Hex\")]",
        )
        .field_attribute(
            "FileManifest.hash",
            "#[serde_as(as = \"serde_with::hex::Hex\")]",
        )
        // Metadata is serialized as a vector of tuples with hex values
        .field_attribute(
            "FileManifest.metadata",
            "#[serde_as(as = \"Vec<(_, serde_with::hex::Hex)>\")]",
        )
        // Chunks are serialized as a vector of hex values
        .field_attribute(
            "FileManifest.chunks",
            "#[serde_as(as = \"Vec<serde_with::hex::Hex>\")]",
        )
        // Path is serialized using a custom path serializer
        .field_attribute(
            "FileManifest.path",
            "#[serde(serialize_with = \"serde_path_serializer::serialize_path\")]",
        )
        // Symlink target is serialized as Base64
        .field_attribute(
            "FileManifest.symlink",
            "#[serde_as(as = \"serde_with::base64::Base64\")]",
        )
        // Compile the proto file
        .compile_protos(&["./woodstock.proto"], &["./"])?;

    // Add special linking for musl targets
    // When compiling for musl (static linking), we need to explicitly
    // link the math library
    if std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "musl" {
        println!("cargo:rustc-link-lib=m");
    }

    Ok(())
}

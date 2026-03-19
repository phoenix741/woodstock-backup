//! Protobuf module.
//!
//! This module provides reusable helpers for reading and writing length-delimited protobuf
//! messages, as well as higher-level readers and writers used throughout the project.
//! Compression-aware file helpers are also re-exported here so callers can interact with a single
//! protobuf-focused module.

/// Handles reading protobuf files.
mod protobuf_reader;
/// Handles writing protobuf files.
mod protobuf_writer;

/// Re-exports of read-side protobuf helpers.
pub use protobuf_reader::{
    read_length_delimited_message, read_optional_length_delimited_message, ProtobufReader,
};
/// Re-exports of write-side protobuf helpers.
pub use protobuf_writer::{
    save_file, write_length_delimited_message, CompressedWriter, ProtobufWriter, UnCompressedWriter,
};

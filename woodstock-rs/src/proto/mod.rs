//! Protobuf module.
//!
//! This module provides utilities for reading and writing protobuf files, including support for compression and atomic writes.

/// Handles reading protobuf files.
mod protobuf_reader;
/// Handles writing protobuf files.
mod protobuf_writer;

pub use protobuf_reader::ProtobufReader;
pub use protobuf_writer::{save_file, CompressedWriter, ProtobufWriter, UnCompressedWriter};

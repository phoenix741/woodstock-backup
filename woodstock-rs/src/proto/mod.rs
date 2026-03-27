//! Protobuf module.
//!
//! This module provides utilities for reading and writing protobuf files, including support for compression and atomic writes.

/// Handles reading protobuf files.
mod protobuf_reader;
/// Handles writing protobuf files.
mod protobuf_writer;

pub use protobuf_reader::{read_length_delimited_message, ProtobufReader};
pub use protobuf_writer::{
    save_file, write_length_delimited_message, CompressedWriter, ProtobufWriter, UnCompressedWriter,
};

//! # Client Module
//!
//! The client module provides the core functionality for the Woodstock backup client, including authentication,
//! configuration, file scanning, server communication, and command execution. This module forms the backbone of the
//! client-side components in the Woodstock backup system.
//!
//! ## Module Structure
//!
//! The client module is organized into several submodules, each responsible for a specific aspect of client
//! functionality:
//!
//! * `authentification` - Handles security and authentication mechanisms
//! * `config` - Manages client configuration settings
//! * `exexcute_command` - Provides command execution capabilities
//! * `resolve` - Implements server discovery methods
//! * `scanner` - Manages file scanning and manifest operations
//! * `server` - Implements the gRPC service for client-server communication
//!
//! ## Usage
//!
//! The client components are typically initialized in the following order:
//!
//! 1. Load client configuration using `config::read_config`
//! 2. Initialize authentication service with `authentification::Service::new`
//! 3. Create a gRPC service instance with `server::WoodstockClient::new`
//! 4. Start the service using Tonic's gRPC server implementation
//!
//! ## Features
//!
//! * Secure authentication using JWT tokens and TLS certificates
//! * Flexible server discovery through mDNS or direct connection
//! * Efficient file scanning with metadata extraction
//! * Incremental backup capabilities
//! * File restoration with metadata preservation
//! * Cross-platform support for Windows and Unix-like systems
//!
//! ## Platform-Specific Functionality
//!
//! The client module contains platform-specific code for handling file metadata, access control lists (ACLs), and
//! extended attributes (xattrs) on different operating systems. These implementations are conditionally compiled
//! based on the target platform.

/// Module for client authentication using JWT tokens and certificate-based verification.
///
/// This module provides secure authentication mechanisms for the client, including:
/// - JWT token generation and validation
/// - Session management with timeouts
/// - Certificate-based authentication
/// - Secure token exchange between client and server
pub mod authentification;

/// Module for client configuration management including settings for hostname, binding, security, etc.
///
/// This module handles all aspects of client configuration, including:
/// - Loading configuration from files or environment variables
/// - Default configuration generation
/// - Configuration validation and normalization
/// - Platform-specific configuration settings
pub mod config;

/// Module providing command execution capabilities for the client.
///
/// This module allows the client to:
/// - Execute shell commands securely
/// - Parse command output
/// - Handle command execution errors
/// - Report command execution status
pub mod exexcute_command;

/// Module implementing server address resolution strategies (mDNS, direct DNS).
///
/// This module provides mechanisms for discovering backup servers through:
/// - Multicast DNS (mDNS) for local network discovery
/// - Direct DNS resolution for known server addresses
/// - Fallback mechanisms when primary resolution fails
pub mod resolve;

/// Module for scanning files and managing file manifests during backup and restore operations.
///
/// This module includes functionality for:
/// - Traversing file systems efficiently
/// - Creating file manifests with metadata
/// - Chunking files for incremental backup
/// - Calculating file and chunk hashes
/// - Managing file restoration with metadata preservation
pub mod scanner;

/// Module implementing the gRPC service for the Woodstock backup client.
///
/// This module provides the RPC implementation that:
/// - Exposes client functionality to the server
/// - Handles authentication and session management
/// - Processes backup and restore requests
/// - Manages streaming of file data and manifests
/// - Handles command execution requests
pub mod server;

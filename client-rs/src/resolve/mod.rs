//! # Resolve Module
//!
//! This module provides mechanisms for discovering and connecting to Woodstock backup servers. It supports multiple
//! discovery methods to ensure flexibility and reliability in various network environments.
//!
//! ## Submodules
//!
//! * `mdns` - Implements multicast DNS-based server discovery
//! * `direct` - Provides direct connection capabilities to specified server addresses
//!
//! ## Features
//!
//! * Automatic server discovery using mDNS
//! * Manual server configuration for direct connections
//! * Support for secure connections using TLS certificates
//!
//! ## Usage
//!
//! Use the `mdns` submodule for automatic discovery in local networks or the `direct` submodule to limit
//! broadcasting.

#[cfg(feature = "mdns")]
/// Implements multicast DNS-based server discovery.
mod mdns;
#[cfg(feature = "mdns")]
pub use mdns::MdnsResolveClient;

/// Provides direct connection capabilities to specified server addresses.
mod direct;
pub use direct::DirectResolveClient;

use eyre::Result;

/// Trait defining the behavior of a client for resolving server addresses.
///
/// This trait is implemented by different resolution strategies like mDNS or direct DNS.
#[tonic::async_trait]
pub trait ResolveClient {
    /// Starts the resolution client.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the client fails to start.
    async fn start(&self) -> Result<()>;

    /// Stops the resolution client.
    async fn stop(&self);

    /// Shuts down the resolution client, releasing any resources.
    async fn shutdown(&self);
}

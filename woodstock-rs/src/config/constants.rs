//! # Constants Module
//!
//! This module defines system-wide constants used throughout the Woodstock backup application.
//! These constants provide consistent values for network services, file operations, and
//! performance tuning parameters.

/// Default port number for Woodstock backup service.
///
/// This port is used for the gRPC communication between backup clients and servers
/// unless explicitly overridden in configuration.
pub const DEFAULT_PORT: u16 = 3657;

/// The full mDNS service name used for service discovery on local networks.
///
/// This constant follows the mDNS naming convention with service type and domain.
pub const MDNS_SERVICE_NAME: &str = "_woodstock._tcp.local.";

/// The suffix part of the mDNS service name.
///
/// This is used for constructing and parsing mDNS responses.
pub const MDNS_SUFFIX: &str = "._woodstock._tcp.local.";

/// The timeout in milliseconds for mDNS discovery operations.
///
/// After this period, if no response is received, the discovery operation will time out.
pub const MDNS_TIMEOUT_MSEC: u64 = 1_000;

/// Size of each chunk in the backup system (16 MB).
///
/// Files larger than this size will be split into multiple chunks during backup.
/// This is used for the backup deduplication strategy.
pub const CHUNK_SIZE: usize = 1 << 24; // 16MB

/// Same as `CHUNK_SIZE` but expressed as a u64 for contexts where u64 is needed.
pub const CHUNK_SIZE_U64: u64 = 1 << 24; // 16MB

/// Size of the buffer used for network file transfers (128 KB).
///
/// This buffer size is optimized for network efficiency and memory usage.
/// Note that `CHUNK_SIZE` should be a multiple of this value.
pub const BUFFER_SIZE: usize = 1 << 17; // 128KB (128 * 128Kb = 16MB)

/// Default size for message channels in backup and restore operations.
///
/// This defines the number of messages that can be buffered in the channel queue
/// before the sender blocks. A value of 100 provides good balance between memory
/// usage and performance.
pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 100;

/// SHA256 hash of an empty string, precomputed as a 32-byte array.
///
/// This is used as a constant reference value for empty content hashing.
pub const SHA256_EMPTYSTRING: [u8; 32] = [
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
    0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
];

/// Redis key used for Woodstock DNS service discovery.
///
/// This key is used in Redis to store and retrieve DNS entries for remote clients.
pub const REDIS_WOODSTOCK_KEY_DNS: &str = "woodstock_dns";

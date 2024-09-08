/// mDNS service name
pub const MDNS_SERVICE_NAME: &str = "_woodstock._tcp.local.";

/// Size of the chunk in the pool directory
pub const CHUNK_SIZE: usize = 1 << 24; // 16MB
pub const CHUNK_SIZE_U64: u64 = 1 << 24; // 16MB

/// Size of the buffer used to transfert file on the network (should be a multiple of `CHUNK_SIZE`)
pub const BUFFER_SIZE: usize = 1 << 17; // 128KB (128 * 128Kb = 16MB)

/// SHA256 of empty string
pub const SHA256_EMPTYSTRING: [u8; 32] = [
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
    0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
];

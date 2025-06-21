use crate::ChunkAlgorithm;

pub const DEFAULT_CHUNK_ALGORITHM: ChunkAlgorithm = ChunkAlgorithm::Blake3;

/// SHA256 hash of an empty string, precomputed as a 32-byte array.
///
/// This is used as a constant reference value for empty content hashing.
pub const SHA256_EMPTYSTRING: [u8; 32] = [
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
    0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
];

/// Blake3 hash of an empty string, precomputed as a 32-byte array.
///
/// This is used as a constant reference value for empty content hashing with Blake3 algorithm.
pub const BLAKE3_EMPTYSTRING: [u8; 32] = [
    0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6, 0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc, 0xc9, 0x49,
    0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7, 0xcc, 0x9a, 0x93, 0xca, 0xe4, 0x1f, 0x32, 0x62,
];

/// SHA3-256 hash of an empty string, precomputed as a 32-byte array.
///
/// This is used as a constant reference value for empty content hashing with SHA3-256 algorithm.
pub const SHA3_EMPTYSTRING: [u8; 32] = [
    0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6, 0x62,
    0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8, 0x43, 0x4a,
];

/// Checks if a hash corresponds to an empty file for the given algorithm.
///
/// # Arguments
/// * `hash` - The hash to check
/// * `algorithm` - The chunk algorithm used
///
/// # Returns
/// * `true` if the hash corresponds to an empty file, `false` otherwise
#[must_use]
pub fn is_empty_hash(hash: &[u8], algorithm: ChunkAlgorithm) -> bool {
    match algorithm {
        ChunkAlgorithm::Blake3 => hash == BLAKE3_EMPTYSTRING,
        ChunkAlgorithm::Sha3256 => hash == SHA3_EMPTYSTRING,
        ChunkAlgorithm::Sha2256 => hash == SHA256_EMPTYSTRING,
    }
}

/// Returns the empty hash for the given algorithm.
///
/// # Arguments
/// * `algorithm` - The chunk algorithm to use
///
/// # Returns
/// * The empty hash as a Vec<u8>
#[must_use]
pub fn get_empty_hash(algorithm: ChunkAlgorithm) -> Vec<u8> {
    match algorithm {
        ChunkAlgorithm::Blake3 => BLAKE3_EMPTYSTRING.to_vec(),
        ChunkAlgorithm::Sha3256 => SHA3_EMPTYSTRING.to_vec(),
        ChunkAlgorithm::Sha2256 => SHA256_EMPTYSTRING.to_vec(),
    }
}

pub trait ChunkHasher {
    fn update(&mut self, data: &[u8]);
    fn finalize(&mut self) -> Vec<u8>;
}

/// Creates a new chunk hasher based on the specified algorithm.
///
/// # Arguments
/// * `algorithm` - The chunk hashing algorithm to use.
///
/// # Returns
///
/// A boxed implementation of the `ChunkHasher` trait.
#[must_use]
pub fn create_chunk_hasher(algorithm: ChunkAlgorithm) -> Box<dyn ChunkHasher + Send + Sync> {
    match algorithm {
        ChunkAlgorithm::Blake3 => Box::new(Blake3ChunkHasher::new()),
        ChunkAlgorithm::Sha3256 => Box::new(Sha3ChunkHasher::new()),
        ChunkAlgorithm::Sha2256 => Box::new(Sha2ChunkHasher::new()),
    }
}

// Implementing blake3

#[derive(Clone)]
pub struct Blake3ChunkHasher {
    /// The optional blake3 hasher instance used for chunk hashing operations.
    hasher: Option<blake3::Hasher>,
}

impl Blake3ChunkHasher {
    /// Creates a new instance of `Blake3ChunkHasher`.
    ///
    /// # Returns
    ///
    /// A new `Blake3ChunkHasher` instance.
    #[must_use]
    pub fn new() -> Self {
        Blake3ChunkHasher {
            hasher: Some(blake3::Hasher::new()),
        }
    }
}

impl Default for Blake3ChunkHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkHasher for Blake3ChunkHasher {
    /// Updates the hasher with the provided data.
    ///
    /// # Arguments
    /// * `data` - The data to hash.
    fn update(&mut self, data: &[u8]) {
        let Some(ref mut hasher) = self.hasher else {
            panic!("Hasher is not initialized");
        };
        hasher.update_rayon(data);
    }

    /// Finalizes the hashing process and returns the hash.
    ///
    /// # Returns
    ///
    /// A vector containing the hash.
    fn finalize(&mut self) -> Vec<u8> {
        let Some(hasher) = self.hasher.take() else {
            panic!("Hasher is not initialized");
        };
        hasher.finalize().as_bytes().to_vec()
    }
}

// Implementing sha3
#[derive(Clone)]
pub struct Sha3ChunkHasher {
    /// The optional SHA3-256 hasher instance used for chunk hashing operations.
    hasher: Option<sha3::Sha3_256>,
}

impl Sha3ChunkHasher {
    /// Creates a new instance of `Sha3ChunkHasher`.
    ///
    /// # Returns
    ///
    /// A new `Sha3ChunkHasher` instance.
    #[must_use]
    pub fn new() -> Self {
        use sha3::{Digest, Sha3_256};

        Sha3ChunkHasher {
            hasher: Some(Sha3_256::new()),
        }
    }
}

impl Default for Sha3ChunkHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkHasher for Sha3ChunkHasher {
    /// Updates the hasher with the provided data.
    ///
    /// # Arguments
    /// * `data` - The data to hash.
    fn update(&mut self, data: &[u8]) {
        use sha3::Digest;

        let Some(ref mut hasher) = self.hasher else {
            panic!("Hasher is not initialized");
        };

        hasher.update(data);
    }

    /// Finalizes the hashing process and returns the hash.
    ///
    /// # Returns
    ///
    /// A vector containing the hash.
    fn finalize(&mut self) -> Vec<u8> {
        let Some(hasher) = self.hasher.take() else {
            panic!("Hasher is not initialized");
        };

        use sha3::Digest;
        hasher.finalize().to_vec()
    }
}

// Implementing Sha2

#[derive(Clone)]
pub struct Sha2ChunkHasher {
    /// The optional SHA2-256 hasher instance used for chunk hashing operations.
    hasher: Option<sha2::Sha256>,
}

impl Sha2ChunkHasher {
    /// Creates a new instance of `Sha2ChunkHasher`.
    ///
    /// # Returns
    ///
    /// A new `Sha2ChunkHasher` instance.
    #[must_use]
    pub fn new() -> Self {
        use sha2::{Digest, Sha256};

        Sha2ChunkHasher {
            hasher: Some(Sha256::new()),
        }
    }
}

impl Default for Sha2ChunkHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkHasher for Sha2ChunkHasher {
    /// Updates the hasher with the provided data.
    ///
    /// # Arguments
    /// * `data` - The data to hash.
    fn update(&mut self, data: &[u8]) {
        use sha2::Digest;

        let Some(ref mut hasher) = self.hasher else {
            panic!("Hasher is not initialized");
        };
        hasher.update(data);
    }

    /// Finalizes the hashing process and returns the hash.
    ///
    /// # Returns
    ///
    /// A vector containing the hash.
    fn finalize(&mut self) -> Vec<u8> {
        use sha2::Digest;

        let Some(hasher) = self.hasher.take() else {
            panic!("Hasher is not initialized");
        };

        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_hash_detection() {
        assert!(is_empty_hash(&BLAKE3_EMPTYSTRING, ChunkAlgorithm::Blake3));
        assert!(is_empty_hash(&SHA256_EMPTYSTRING, ChunkAlgorithm::Sha2256));
        assert!(is_empty_hash(&SHA3_EMPTYSTRING, ChunkAlgorithm::Sha3256));
        assert!(!is_empty_hash(&[0u8; 32], ChunkAlgorithm::Blake3));
        assert!(!is_empty_hash(&[0u8; 32], ChunkAlgorithm::Sha2256));
        assert!(!is_empty_hash(&[0u8; 32], ChunkAlgorithm::Sha3256));
    }

    #[test]
    fn test_get_empty_hash() {
        assert_eq!(
            get_empty_hash(ChunkAlgorithm::Blake3),
            BLAKE3_EMPTYSTRING.to_vec()
        );
        assert_eq!(
            get_empty_hash(ChunkAlgorithm::Sha2256),
            SHA256_EMPTYSTRING.to_vec()
        );
        assert_eq!(
            get_empty_hash(ChunkAlgorithm::Sha3256),
            SHA3_EMPTYSTRING.to_vec()
        );
    }

    #[test]
    fn test_empty_hash_consistency() {
        // Test that hashing empty data produces the expected empty hash
        let mut blake3_hasher = Blake3ChunkHasher::new();
        let blake3_result = blake3_hasher.finalize();
        assert_eq!(blake3_result, BLAKE3_EMPTYSTRING.to_vec());

        let mut sha2_hasher = Sha2ChunkHasher::new();
        let sha2_result = sha2_hasher.finalize();
        assert_eq!(sha2_result, SHA256_EMPTYSTRING.to_vec());

        let mut sha3_hasher = Sha3ChunkHasher::new();
        let sha3_result = sha3_hasher.finalize();
        assert_eq!(sha3_result, SHA3_EMPTYSTRING.to_vec());
    }
}

use crate::ChunkAlgorithm;

pub const DEFAULT_CHUNK_ALGORITHM: ChunkAlgorithm = ChunkAlgorithm::Blake3;

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

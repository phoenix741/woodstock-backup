use crate::ChunkAlgorithm;

pub const DEFAULT_CHUNK_ALGORITHM: ChunkAlgorithm = ChunkAlgorithm::Blake3;

pub trait ChunkHasher {
    fn update(&mut self, data: &[u8]);
    fn finalize(&mut self) -> Vec<u8>;
}

#[must_use]
pub fn create_chunk_hasher(algorithm: &ChunkAlgorithm) -> Box<dyn ChunkHasher + Send + Sync> {
    match algorithm {
        ChunkAlgorithm::Blake3 => Box::new(Blake3ChunkHasher::new()),
        ChunkAlgorithm::Sha3256 => Box::new(Sha3ChunkHasher::new()),
        ChunkAlgorithm::Sha2256 => Box::new(Sha2ChunkHasher::new()),
    }
}

// Implementing blake3

#[derive(Clone)]
pub struct Blake3ChunkHasher {
    hasher: Option<blake3::Hasher>,
}

impl Blake3ChunkHasher {
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
    fn update(&mut self, data: &[u8]) {
        let Some(ref mut hasher) = self.hasher else {
            panic!("Hasher is not initialized");
        };
        hasher.update_rayon(data);
    }

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
    hasher: Option<sha3::Sha3_256>,
}

impl Sha3ChunkHasher {
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
    fn update(&mut self, data: &[u8]) {
        let Some(ref mut hasher) = self.hasher else {
            panic!("Hasher is not initialized");
        };

        use sha3::Digest;
        hasher.update(data);
    }

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
    hasher: Option<sha2::Sha256>,
}

impl Sha2ChunkHasher {
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
    fn update(&mut self, data: &[u8]) {
        let Some(ref mut hasher) = self.hasher else {
            panic!("Hasher is not initialized");
        };
        use sha2::Digest;
        hasher.update(data);
    }

    fn finalize(&mut self) -> Vec<u8> {
        let Some(hasher) = self.hasher.take() else {
            panic!("Hasher is not initialized");
        };

        use sha2::Digest;
        hasher.finalize().to_vec()
    }
}

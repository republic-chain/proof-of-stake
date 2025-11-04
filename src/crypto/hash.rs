use crate::types::Hash;
use sha2::{Digest, Sha256};

pub struct Hasher;

impl Hasher {
    pub fn hash(data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    pub fn hash_multiple(data_chunks: &[&[u8]]) -> Hash {
        let mut hasher = Sha256::new();
        for chunk in data_chunks {
            hasher.update(chunk);
        }
        hasher.finalize().into()
    }

    pub fn hash_two(left: &Hash, right: &Hash) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    pub fn double_hash(data: &[u8]) -> Hash {
        let first_hash = Self::hash(data);
        Self::hash(&first_hash)
    }

    pub fn alternative_hash(data: &[u8]) -> Hash {
        // Alternative hash function using double SHA256
        let first = Self::hash(data);
        Self::hash(&first)
    }

    pub fn hash_with_domain(domain: &[u8], data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(domain);
        hasher.update(data);
        hasher.finalize().into()
    }

    pub fn hash_serializable<T: serde::Serialize>(data: &T) -> Result<Hash, serde_json::Error> {
        let serialized = serde_json::to_vec(data)?;
        Ok(Self::hash(&serialized))
    }
}

pub struct HashBuilder {
    hasher: Sha256,
}

impl HashBuilder {
    pub fn new() -> Self {
        HashBuilder {
            hasher: Sha256::new(),
        }
    }

    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.hasher.update(data);
        self
    }

    pub fn update_u64(&mut self, value: u64) -> &mut Self {
        self.hasher.update(value.to_le_bytes());
        self
    }

    pub fn update_u32(&mut self, value: u32) -> &mut Self {
        self.hasher.update(value.to_le_bytes());
        self
    }

    pub fn update_hash(&mut self, hash: &Hash) -> &mut Self {
        self.hasher.update(hash);
        self
    }

    pub fn finalize(self) -> Hash {
        self.hasher.finalize().into()
    }
}

impl Default for HashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compute_domain(domain_type: &[u8; 4], fork_version: &[u8; 4], genesis_validators_root: &Hash) -> Hash {
    let mut fork_data_root = HashBuilder::new();
    fork_data_root
        .update(fork_version)
        .update(genesis_validators_root);

    let fork_data_root = fork_data_root.finalize();

    let mut domain = HashBuilder::new();
    domain
        .update(domain_type)
        .update(&fork_data_root[..28]); // Take first 28 bytes

    domain.finalize()
}

pub fn compute_signing_root(object_root: &Hash, domain: &Hash) -> Hash {
    let mut signing_root = HashBuilder::new();
    signing_root
        .update(object_root)
        .update(domain);

    signing_root.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let data = b"hello world";
        let hash1 = Hasher::hash(data);
        let hash2 = Hasher::hash(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_hash_builder() {
        let data1 = b"hello";
        let data2 = b"world";

        let hash1 = {
            let mut builder = HashBuilder::new();
            builder.update(data1).update(data2);
            builder.finalize()
        };

        let hash2 = Hasher::hash_multiple(&[data1, data2]);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_two() {
        let hash1 = Hasher::hash(b"left");
        let hash2 = Hasher::hash(b"right");

        let combined = Hasher::hash_two(&hash1, &hash2);
        assert_eq!(combined.len(), 32);
    }

    #[test]
    fn test_double_hash() {
        let data = b"test data";
        let single = Hasher::hash(data);
        let double = Hasher::double_hash(data);

        assert_ne!(single, double);
        assert_eq!(double, Hasher::hash(&single));
    }
}
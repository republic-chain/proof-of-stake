pub mod keys;
pub mod signatures;
pub mod hash;
pub mod merkle;

pub use keys::*;
pub use signatures::*;
pub use hash::*;
pub use merkle::*;

use crate::types::{Hash, Signature, PublicKey, PrivateKey};
use anyhow::Result;
use rand::rngs::OsRng;

pub struct CryptoProvider;

impl CryptoProvider {
    pub fn generate_keypair() -> (PrivateKey, PublicKey) {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        (signing_key.to_bytes(), verifying_key.to_bytes())
    }

    pub fn sign(private_key: &PrivateKey, message: &[u8]) -> Result<Signature> {
        use ed25519_dalek::Signer;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(private_key);
        let signature = signing_key.sign(message);
        Ok(Signature(signature.to_bytes()))
    }

    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> Result<()> {
        use ed25519_dalek::Verifier;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(public_key)?;
        let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
        verifying_key.verify(message, &sig)?;
        Ok(())
    }

    pub fn hash(data: &[u8]) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    pub fn hash_two(left: &Hash, right: &Hash) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    pub fn generate_random_bytes(size: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; size];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    pub fn generate_random_hash() -> Hash {
        let bytes = Self::generate_random_bytes(32);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        hash
    }
}
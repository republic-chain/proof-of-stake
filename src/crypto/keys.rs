use crate::types::{Address, PublicKey, PrivateKey};
use anyhow::{Result, anyhow};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
}

impl KeyPair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let private_key = signing_key.to_bytes();
        let public_key = verifying_key.to_bytes();
        let address = Address::from(public_key);

        KeyPair {
            private_key,
            public_key,
            address,
        }
    }

    pub fn from_private_key(private_key: PrivateKey) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(&private_key);
        let verifying_key = signing_key.verifying_key();
        let public_key = verifying_key.to_bytes();
        let address = Address::from(public_key);

        Ok(KeyPair {
            private_key,
            public_key,
            address,
        })
    }

    pub fn from_hex(hex_private_key: &str) -> Result<Self> {
        let bytes = hex::decode(hex_private_key)
            .map_err(|e| anyhow!("Invalid hex string: {}", e))?;

        if bytes.len() != 32 {
            return Err(anyhow!("Private key must be 32 bytes"));
        }

        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&bytes);

        Self::from_private_key(private_key)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.private_key)
    }

    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.private_key)
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey::from_bytes(&self.public_key).expect("Valid public key")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: Address,
    pub public_key: PublicKey,
}

impl From<&KeyPair> for WalletInfo {
    fn from(keypair: &KeyPair) -> Self {
        WalletInfo {
            address: keypair.address,
            public_key: keypair.public_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.public_key.len(), 32);
    }

    #[test]
    fn test_keypair_from_private_key() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::from_private_key(keypair1.private_key).unwrap();

        assert_eq!(keypair1.private_key, keypair2.private_key);
        assert_eq!(keypair1.public_key, keypair2.public_key);
        assert_eq!(keypair1.address, keypair2.address);
    }

    #[test]
    fn test_keypair_hex_conversion() {
        let keypair1 = KeyPair::generate();
        let hex = keypair1.to_hex();
        let keypair2 = KeyPair::from_hex(&hex).unwrap();

        assert_eq!(keypair1.private_key, keypair2.private_key);
        assert_eq!(keypair1.public_key, keypair2.public_key);
        assert_eq!(keypair1.address, keypair2.address);
    }
}
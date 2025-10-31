pub mod block;
pub mod transaction;
pub mod validator;
pub mod account;
pub mod consensus;

pub use block::*;
pub use transaction::*;
pub use validator::*;
pub use account::*;
pub use consensus::*;

use serde::{Deserialize, Serialize};
use std::fmt;

pub type Hash = [u8; 32];
pub type PublicKey = [u8; 32];
pub type PrivateKey = [u8; 32];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes);
        Ok(Signature(array))
    }
}
pub type Amount = u64;
pub type Nonce = u64;
pub type Slot = u64;
pub type Epoch = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub Hash);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<PublicKey> for Address {
    fn from(public_key: PublicKey) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        Address(hasher.finalize().into())
    }
}

impl From<&str> for Address {
    fn from(s: &str) -> Self {
        let bytes = hex::decode(s).expect("Invalid hex string");
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes[..32]);
        Address(array)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkId {
    Mainnet = 1,
    Testnet = 2,
    Devnet = 3,
}

impl Default for NetworkId {
    fn default() -> Self {
        NetworkId::Devnet
    }
}
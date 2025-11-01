use crate::types::{Signature, PublicKey, Hash};
use anyhow::{Result, anyhow};
use ed25519_dalek::{SigningKey, VerifyingKey};

pub struct SignatureUtils;

impl SignatureUtils {
    pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
        use ed25519_dalek::Signer;
        let signature = signing_key.sign(message);
        Signature(signature.to_bytes())
    }

    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> Result<()> {
        use ed25519_dalek::Verifier;
        let verifying_key = VerifyingKey::from_bytes(public_key)
            .map_err(|e| anyhow!("Invalid public key: {}", e))?;

        let sig = ed25519_dalek::Signature::from_bytes(&signature.0);

        verifying_key.verify(message, &sig)
            .map_err(|e| anyhow!("Signature verification failed: {}", e))?;

        Ok(())
    }

    pub fn aggregate_signatures(signatures: &[Signature]) -> Result<Signature> {
        // For production use, you would implement BLS signature aggregation
        // For now, this is a placeholder that just returns the first signature
        if signatures.is_empty() {
            return Err(anyhow!("No signatures to aggregate"));
        }

        // This is a simplified implementation
        // In a real system, you would use BLS signatures for aggregation
        Ok(signatures[0])
    }

    pub fn verify_aggregated(
        public_keys: &[PublicKey],
        messages: &[&[u8]],
        aggregated_signature: &Signature,
    ) -> Result<()> {
        // Placeholder for BLS aggregate signature verification
        // For now, just verify the first signature with the first key and message
        if public_keys.is_empty() || messages.is_empty() {
            return Err(anyhow!("Empty keys or messages"));
        }

        Self::verify(&public_keys[0], messages[0], aggregated_signature)
    }

    pub fn sign_hash(signing_key: &SigningKey, hash: &Hash) -> Signature {
        Self::sign(signing_key, hash)
    }

    pub fn verify_hash(public_key: &PublicKey, hash: &Hash, signature: &Signature) -> Result<()> {
        Self::verify(public_key, hash, signature)
    }

    pub fn batch_verify(
        public_keys: &[PublicKey],
        messages: &[&[u8]],
        signatures: &[Signature],
    ) -> Result<()> {
        if public_keys.len() != messages.len() || messages.len() != signatures.len() {
            return Err(anyhow!("Mismatched lengths"));
        }

        for ((public_key, message), signature) in
            public_keys.iter().zip(messages.iter()).zip(signatures.iter()) {
            Self::verify(public_key, message, signature)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MultiSignature {
    pub signatures: Vec<Signature>,
    pub signers: Vec<PublicKey>,
    pub threshold: usize,
}

impl MultiSignature {
    pub fn new(threshold: usize) -> Self {
        MultiSignature {
            signatures: Vec::new(),
            signers: Vec::new(),
            threshold,
        }
    }

    pub fn add_signature(&mut self, signature: Signature, signer: PublicKey) -> Result<()> {
        if self.signers.contains(&signer) {
            return Err(anyhow!("Signer already added"));
        }

        self.signatures.push(signature);
        self.signers.push(signer);

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.signatures.len() >= self.threshold &&
        self.signatures.len() == self.signers.len()
    }

    pub fn verify(&self, message: &[u8]) -> Result<()> {
        if !self.is_valid() {
            return Err(anyhow!("Insufficient signatures"));
        }

        for (signature, signer) in self.signatures.iter().zip(self.signers.iter()) {
            SignatureUtils::verify(signer, message, signature)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate();
        let message = b"test message";

        let signature = SignatureUtils::sign(&keypair.signing_key(), message);
        let result = SignatureUtils::verify(&keypair.public_key, message, &signature);

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();
        let message = b"test message";

        let signature = SignatureUtils::sign(&keypair1.signing_key(), message);
        let result = SignatureUtils::verify(&keypair2.public_key, message, &signature);

        assert!(result.is_err());
    }

    #[test]
    fn test_multi_signature() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();
        let message = b"test message";

        let sig1 = SignatureUtils::sign(&keypair1.signing_key(), message);
        let sig2 = SignatureUtils::sign(&keypair2.signing_key(), message);

        let mut multi_sig = MultiSignature::new(2);
        multi_sig.add_signature(sig1, keypair1.public_key).unwrap();
        multi_sig.add_signature(sig2, keypair2.public_key).unwrap();

        assert!(multi_sig.is_valid());
        assert!(multi_sig.verify(message).is_ok());
    }
}
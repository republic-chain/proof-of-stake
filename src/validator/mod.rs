// Validator module - validator operations and management

use crate::types::*;
use crate::crypto::*;

pub struct ValidatorService {
    keypair: Option<KeyPair>,
    is_active: bool,
}

impl ValidatorService {
    pub fn new() -> Self {
        ValidatorService {
            keypair: None,
            is_active: false,
        }
    }

    pub fn load_keypair(&mut self, private_key: PrivateKey) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = KeyPair::from_private_key(private_key)?;
        self.keypair = Some(keypair);
        Ok(())
    }

    pub fn start_validating(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.keypair.is_none() {
            return Err("No keypair loaded".into());
        }
        self.is_active = true;
        Ok(())
    }

    pub fn stop_validating(&mut self) {
        self.is_active = false;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn get_address(&self) -> Option<Address> {
        self.keypair.as_ref().map(|kp| kp.address)
    }

    pub fn sign_block(&self, block: &mut Block) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(keypair) = &self.keypair {
            block.sign(&keypair.signing_key());
            Ok(())
        } else {
            Err("No keypair available for signing".into())
        }
    }

    pub fn create_attestation(&self, slot: Slot, beacon_block_root: Hash) -> Result<Attestation, Box<dyn std::error::Error>> {
        let _keypair = self.keypair.as_ref().ok_or("No keypair available")?;

        let attestation = Attestation {
            slot,
            beacon_block_root,
            source_epoch: 0, // TODO: Get from consensus state
            source_root: [0u8; 32],
            target_epoch: slot / 32, // Assuming 32 slots per epoch
            target_root: beacon_block_root,
            validator_index: 0, // TODO: Get validator index
            signature: Signature([0u8; 64]), // Will be filled by signing
        };

        // TODO: Sign the attestation
        Ok(attestation)
    }
}

impl Default for ValidatorService {
    fn default() -> Self {
        Self::new()
    }
}
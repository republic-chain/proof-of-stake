// Storage module - placeholder for database implementation
// TODO: Implement SQLite-based storage

use crate::types::*;
use std::collections::HashMap;

pub struct StorageService {
    // In-memory storage for now - would be replaced with SQLite
    blocks: HashMap<Hash, Block>,
    accounts: HashMap<Address, Account>,
    validators: HashMap<Address, Validator>,
    latest_height: u64,
}

impl StorageService {
    pub fn new() -> Self {
        StorageService {
            blocks: HashMap::new(),
            accounts: HashMap::new(),
            validators: HashMap::new(),
            latest_height: 0,
        }
    }

    pub async fn store_block(&mut self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let hash = block.hash();
        self.latest_height = block.header.height;
        self.blocks.insert(hash, block);
        Ok(())
    }

    pub async fn get_block(&self, hash: &Hash) -> Result<Option<Block>, Box<dyn std::error::Error>> {
        Ok(self.blocks.get(hash).cloned())
    }

    pub async fn get_latest_height(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(self.latest_height)
    }

    pub async fn store_account(&mut self, account: Account) -> Result<(), Box<dyn std::error::Error>> {
        self.accounts.insert(account.address, account);
        Ok(())
    }

    pub async fn get_account(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        Ok(self.accounts.get(address).cloned())
    }

    pub async fn store_validator(&mut self, validator: Validator) -> Result<(), Box<dyn std::error::Error>> {
        self.validators.insert(validator.address, validator);
        Ok(())
    }

    pub async fn get_validator(&self, address: &Address) -> Result<Option<Validator>, Box<dyn std::error::Error>> {
        Ok(self.validators.get(address).cloned())
    }
}

impl Default for StorageService {
    fn default() -> Self {
        Self::new()
    }
}
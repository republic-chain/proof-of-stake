pub mod types;
pub mod crypto;
pub mod consensus;
pub mod network;
pub mod storage;
pub mod validator;
pub mod config;

pub use types::*;
pub use crypto::*;
pub use consensus::*;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Node {
    pub config: config::NodeConfig,
    pub consensus: consensus::ConsensusEngine,
    // Network and storage components would be added here
}

impl Node {
    pub async fn new(config: config::NodeConfig) -> Result<Self> {
        let consensus_config = ConsensusConfig::default();
        let genesis_validators = Vec::new(); // Would load from genesis

        let consensus = ConsensusEngine::new(consensus_config, genesis_validators)?;

        Ok(Node {
            config,
            consensus,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting node with config: {:?}", self.config);

        // Initialize network
        // Initialize storage
        // Start consensus engine
        // Start API server

        Ok(())
    }

    pub fn process_block(&mut self, block: Block) -> Result<()> {
        self.consensus.process_block(&block)
    }

    pub fn get_head(&self) -> Option<Hash> {
        self.consensus.get_head()
    }
}
use crate::types::*;
use std::collections::{HashMap, HashSet};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ForkChoice {
    pub blocks: HashMap<Hash, Block>,
    pub votes: HashMap<Hash, u64>, // block_hash -> vote_weight
    pub latest_messages: HashMap<u64, Hash>, // validator_index -> latest_vote
    pub justified_checkpoint: Checkpoint,
    pub finalized_checkpoint: Checkpoint,
    pub proposer_boost_root: Option<Hash>,
}

impl ForkChoice {
    pub fn new() -> Self {
        ForkChoice {
            blocks: HashMap::new(),
            votes: HashMap::new(),
            latest_messages: HashMap::new(),
            justified_checkpoint: Checkpoint {
                epoch: 0,
                root: [0u8; 32],
            },
            finalized_checkpoint: Checkpoint {
                epoch: 0,
                root: [0u8; 32],
            },
            proposer_boost_root: None,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        let block_hash = block.hash();
        self.blocks.insert(block_hash, block);

        // Apply proposer boost to new block
        self.proposer_boost_root = Some(block_hash);
    }

    pub fn add_attestation(&mut self, attestation: Attestation) {
        let validator_index = attestation.validator_index;
        let target_root = attestation.target_root;

        // Check if this is a new vote from this validator
        if let Some(previous_vote) = self.latest_messages.get(&validator_index) {
            if *previous_vote == target_root {
                return; // Same vote, ignore
            }

            // Remove previous vote weight
            if let Some(weight) = self.votes.get_mut(previous_vote) {
                *weight = weight.saturating_sub(1);
            }
        }

        // Add new vote
        self.latest_messages.insert(validator_index, target_root);
        *self.votes.entry(target_root).or_insert(0) += 1;
    }

    pub fn get_head(&self) -> Option<Hash> {
        if self.blocks.is_empty() {
            return None;
        }

        // Start from finalized checkpoint
        let mut current_root = self.finalized_checkpoint.root;

        // Find the best child at each level
        loop {
            let children = self.get_children(&current_root);
            if children.is_empty() {
                break;
            }

            // Select child with highest weight
            let best_child = children
                .into_iter()
                .max_by_key(|&child_root| self.get_weight(child_root))
                .unwrap();

            current_root = best_child;
        }

        Some(current_root)
    }

    fn get_children(&self, parent_root: &Hash) -> Vec<Hash> {
        self.blocks
            .iter()
            .filter(|(_, block)| block.header.previous_hash == *parent_root)
            .map(|(hash, _)| *hash)
            .collect()
    }

    fn get_weight(&self, block_root: Hash) -> u64 {
        let mut weight = self.votes.get(&block_root).copied().unwrap_or(0);

        // Apply proposer boost
        if Some(block_root) == self.proposer_boost_root {
            weight += 100; // Boost value
        }

        // Add weight from descendants
        for child_root in self.get_children(&block_root) {
            weight += self.get_weight(child_root);
        }

        weight
    }

    pub fn update_justified_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<()> {
        // Validate that the new justified checkpoint is newer
        if checkpoint.epoch <= self.justified_checkpoint.epoch {
            return Err(anyhow::anyhow!("Justified checkpoint epoch must be newer"));
        }

        self.justified_checkpoint = checkpoint;
        Ok(())
    }

    pub fn update_finalized_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<()> {
        // Validate that the new finalized checkpoint is newer
        if checkpoint.epoch <= self.finalized_checkpoint.epoch {
            return Err(anyhow::anyhow!("Finalized checkpoint epoch must be newer"));
        }

        // Finalized checkpoint must be justified
        if checkpoint.epoch > self.justified_checkpoint.epoch {
            return Err(anyhow::anyhow!("Cannot finalize unjustified checkpoint"));
        }

        self.finalized_checkpoint = checkpoint;

        // Prune blocks that are not descendants of the finalized checkpoint
        self.prune_finalized_blocks();

        Ok(())
    }

    fn prune_finalized_blocks(&mut self) {
        let finalized_root = self.finalized_checkpoint.root;
        let mut to_keep = HashSet::new();

        // Find all blocks that are descendants of the finalized block
        self.find_descendants(&finalized_root, &mut to_keep);
        to_keep.insert(finalized_root);

        // Remove blocks that are not descendants
        self.blocks.retain(|hash, _| to_keep.contains(hash));

        // Clean up votes for pruned blocks
        self.votes.retain(|hash, _| to_keep.contains(hash));

        // Clean up latest messages that point to pruned blocks
        self.latest_messages.retain(|_, hash| to_keep.contains(hash));
    }

    fn find_descendants(&self, root: &Hash, descendants: &mut HashSet<Hash>) {
        for child_root in self.get_children(root) {
            descendants.insert(child_root);
            self.find_descendants(&child_root, descendants);
        }
    }

    pub fn get_ancestor(&self, root: Hash, slot: Slot) -> Option<Hash> {
        let mut current_root = root;

        loop {
            let block = self.blocks.get(&current_root)?;
            if block.header.slot <= slot {
                return Some(current_root);
            }

            current_root = block.header.previous_hash;
            if current_root == [0u8; 32] {
                return None;
            }
        }
    }

    pub fn is_descendant(&self, ancestor: Hash, descendant: Hash) -> bool {
        if ancestor == descendant {
            return true;
        }

        let mut current = descendant;

        loop {
            let block = match self.blocks.get(&current) {
                Some(block) => block,
                None => return false,
            };

            if block.header.previous_hash == ancestor {
                return true;
            }

            if block.header.previous_hash == [0u8; 32] {
                return false;
            }

            current = block.header.previous_hash;
        }
    }

    pub fn get_block(&self, hash: &Hash) -> Option<&Block> {
        self.blocks.get(hash)
    }

    pub fn has_block(&self, hash: &Hash) -> bool {
        self.blocks.contains_key(hash)
    }

    pub fn get_chain_length(&self, head: Hash) -> u64 {
        let mut length = 0;
        let mut current = head;

        while let Some(block) = self.blocks.get(&current) {
            length += 1;
            if block.header.previous_hash == [0u8; 32] {
                break;
            }
            current = block.header.previous_hash;
        }

        length
    }

    pub fn clear_proposer_boost(&mut self) {
        self.proposer_boost_root = None;
    }
}

impl Default for ForkChoice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Block, Address};

    fn create_test_block(height: u64, previous_hash: Hash) -> Block {
        let mut block = Block::default();
        block.header.height = height;
        block.header.previous_hash = previous_hash;
        block.header.proposer = Address([0u8; 32]);
        block
    }

    #[test]
    fn test_fork_choice_single_block() {
        let mut fork_choice = ForkChoice::new();
        let block = create_test_block(1, [0u8; 32]);
        let block_hash = block.hash();

        fork_choice.add_block(block);

        assert_eq!(fork_choice.get_head(), Some(block_hash));
    }

    #[test]
    fn test_fork_choice_linear_chain() {
        let mut fork_choice = ForkChoice::new();

        let block1 = create_test_block(1, [0u8; 32]);
        let block1_hash = block1.hash();
        fork_choice.add_block(block1);

        let block2 = create_test_block(2, block1_hash);
        let block2_hash = block2.hash();
        fork_choice.add_block(block2);

        assert_eq!(fork_choice.get_head(), Some(block2_hash));
    }

    #[test]
    fn test_is_descendant() {
        let mut fork_choice = ForkChoice::new();

        let block1 = create_test_block(1, [0u8; 32]);
        let block1_hash = block1.hash();
        fork_choice.add_block(block1);

        let block2 = create_test_block(2, block1_hash);
        let block2_hash = block2.hash();
        fork_choice.add_block(block2);

        assert!(fork_choice.is_descendant(block1_hash, block2_hash));
        assert!(!fork_choice.is_descendant(block2_hash, block1_hash));
        assert!(fork_choice.is_descendant(block1_hash, block1_hash));
    }
}
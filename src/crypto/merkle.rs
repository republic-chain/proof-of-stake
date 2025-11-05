use crate::types::Hash;
use crate::crypto::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleTree {
    pub root: Hash,
    pub leaves: Vec<Hash>,
    pub levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<Hash>) -> Self {
        if leaves.is_empty() {
            return MerkleTree {
                root: [0u8; 32],
                leaves: Vec::new(),
                levels: Vec::new(),
            };
        }

        let mut levels = vec![leaves.clone()];
        let mut current_level = leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let hash = if chunk.len() == 2 {
                    Hasher::hash_two(&chunk[0], &chunk[1])
                } else {
                    // Odd number of nodes, duplicate the last one
                    Hasher::hash_two(&chunk[0], &chunk[0])
                };
                next_level.push(hash);
            }

            levels.push(next_level.clone());
            current_level = next_level;
        }

        let root = current_level[0];

        MerkleTree {
            root,
            leaves,
            levels,
        }
    }

    pub fn from_data<T: AsRef<[u8]>>(data: &[T]) -> Self {
        let leaves: Vec<Hash> = data.iter().map(|item| Hasher::hash(item.as_ref())).collect();
        Self::new(leaves)
    }

    pub fn get_proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level.len() {
                proof.push(MerkleProofElement {
                    hash: level[sibling_index],
                    is_left: current_index % 2 != 0,
                });
            }

            current_index /= 2;
        }

        Some(MerkleProof {
            leaf_hash: self.leaves[index],
            proof,
            root: self.root,
        })
    }

    pub fn verify_proof(proof: &MerkleProof) -> bool {
        let mut current_hash = proof.leaf_hash;

        for element in &proof.proof {
            current_hash = if element.is_left {
                Hasher::hash_two(&element.hash, &current_hash)
            } else {
                Hasher::hash_two(&current_hash, &element.hash)
            };
        }

        current_hash == proof.root
    }

    pub fn update_leaf(&mut self, index: usize, new_leaf: Hash) -> Result<(), String> {
        if index >= self.leaves.len() {
            return Err("Index out of bounds".to_string());
        }

        self.leaves[index] = new_leaf;
        *self = Self::new(self.leaves.clone());
        Ok(())
    }

    pub fn add_leaf(&mut self, leaf: Hash) {
        self.leaves.push(leaf);
        *self = Self::new(self.leaves.clone());
    }

    pub fn size(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_hash: Hash,
    pub proof: Vec<MerkleProofElement>,
    pub root: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProofElement {
    pub hash: Hash,
    pub is_left: bool,
}

impl MerkleProof {
    pub fn verify(&self) -> bool {
        MerkleTree::verify_proof(self)
    }

    pub fn verify_with_root(&self, expected_root: &Hash) -> bool {
        self.root == *expected_root && self.verify()
    }
}

pub struct SparseMerkleTree {
    pub root: Hash,
    pub depth: usize,
    pub nodes: std::collections::HashMap<(usize, u64), Hash>,
    pub default_hashes: Vec<Hash>,
}

impl SparseMerkleTree {
    pub fn new(depth: usize) -> Self {
        let mut default_hashes = vec![[0u8; 32]; depth + 1];

        // Calculate default hashes bottom-up
        for i in (0..depth).rev() {
            default_hashes[i] = Hasher::hash_two(&default_hashes[i + 1], &default_hashes[i + 1]);
        }

        SparseMerkleTree {
            root: default_hashes[0],
            depth,
            nodes: std::collections::HashMap::new(),
            default_hashes,
        }
    }

    pub fn update(&mut self, index: u64, value: Hash) {
        self.update_recursive(0, 0, 1u64 << self.depth, index, value);
    }

    fn update_recursive(&mut self, level: usize, start: u64, size: u64, index: u64, value: Hash) {
        if size == 1 {
            self.nodes.insert((level, start), value);
            return;
        }

        let mid = start + size / 2;
        if index < mid {
            self.update_recursive(level + 1, start, size / 2, index, value);
        } else {
            self.update_recursive(level + 1, mid, size / 2, index, value);
        }

        let left = self.get_node(level + 1, start);
        let right = self.get_node(level + 1, mid);
        let new_hash = Hasher::hash_two(&left, &right);

        self.nodes.insert((level, start), new_hash);

        if level == 0 {
            self.root = new_hash;
        }
    }

    fn get_node(&self, level: usize, index: u64) -> Hash {
        self.nodes
            .get(&(level, index))
            .copied()
            .unwrap_or(self.default_hashes[level])
    }

    pub fn get_proof(&self, index: u64) -> Vec<Hash> {
        let mut proof = Vec::new();
        let mut current_index = index;

        for level in (1..=self.depth).rev() {
            let sibling_index = current_index ^ 1;
            proof.push(self.get_node(level, sibling_index));
            current_index /= 2;
        }

        proof
    }

    pub fn verify_proof(&self, index: u64, value: Hash, proof: &[Hash]) -> bool {
        let mut current_hash = value;
        let mut current_index = index;

        for &sibling_hash in proof {
            current_hash = if current_index % 2 == 0 {
                Hasher::hash_two(&current_hash, &sibling_hash)
            } else {
                Hasher::hash_two(&sibling_hash, &current_hash)
            };
            current_index /= 2;
        }

        current_hash == self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_creation() {
        let data = vec![b"a", b"b", b"c", b"d"];
        let tree = MerkleTree::from_data(&data);

        assert_eq!(tree.leaves.len(), 4);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_merkle_proof() {
        let data = vec![b"a", b"b", b"c", b"d"];
        let tree = MerkleTree::from_data(&data);

        let proof = tree.get_proof(1).unwrap();
        assert!(proof.verify());
        assert!(proof.verify_with_root(&tree.root));
    }

    #[test]
    fn test_sparse_merkle_tree() {
        let mut smt = SparseMerkleTree::new(8);
        let value = Hasher::hash(b"test");

        smt.update(5, value);
        let proof = smt.get_proof(5);

        assert!(smt.verify_proof(5, value, &proof));
    }

    #[test]
    fn test_empty_merkle_tree() {
        let tree = MerkleTree::new(vec![]);
        assert!(tree.is_empty());
        assert_eq!(tree.root, [0u8; 32]);
    }
}
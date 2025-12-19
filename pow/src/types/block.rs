use core::fmt;

use bincode::config;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::types::{subtypes::Hash256, transaction::Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    //header 
    pub timestamp: Option<DateTime<Local>>,
    pub previous_hash: Option<Hash256>, // the only time it doesn't have any previous hash is for the genesis block
    pub hash: Hash256,
    pub nonce: u64,
    
    //body
    pub merkle_root: Hash256,
    transactions: Vec<Transaction>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block {{\n  hash: {:?},\n  previous_hash: {:?}\n}}",
            self.hash,
            self.previous_hash
        )
    }
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, previous_hash: Option<Hash256>) -> Self {
        Self {
            timestamp: None,
            previous_hash: previous_hash,
            hash: Hash256([0; 32]),
            nonce: 0,
            transactions: transactions.clone(),
            merkle_root: Block::merkle_root_from_txs(&transactions),
        }
    }

    pub fn calculate_hash(&self) -> Hash256 {
        let mut raw = String::new();
        for tx in &self.transactions {
            raw.push_str(&format!("{}", tx));
        }
        raw.push_str(&format!("{}", self.nonce));

        match self.previous_hash {
            Some(previous_hash) => raw.push_str(&format!("{:?}", previous_hash)),
            None => (),
        }

        let hasher = Sha256::digest(raw);
        let hash = hasher.as_chunks::<32>().0[0];

        Hash256(hash)
    }

    pub fn hash_transaction(tx: &Transaction) -> Hash256 {
        let serialized = bincode::encode_to_vec(tx, config::standard())
            .expect("Failed to serialize transaction");

        let hash = Sha256::digest(&serialized);

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);

        Hash256(bytes)
    }

    pub fn merkle_root_from_txs(txs: &[Transaction]) -> Hash256 {
        if txs.is_empty() {
            return Hash256([0u8; 32]); // empty tree root
        }

        // Leaf hashes
        let mut hashes: Vec<Hash256> =
            txs.iter().map(Self::hash_transaction).collect();

        // Build Merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::with_capacity((hashes.len() + 1) / 2);

            for pair in hashes.chunks(2) {
                let left = pair[0];
                let right = if pair.len() == 2 {
                    pair[1]
                } else {
                    pair[0] // duplicate last if odd
                };

                let mut hasher = Sha256::new();
                hasher.update(left.0);
                hasher.update(right.0);

                let hash = hasher.finalize();
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(&hash);

                next_level.push(Hash256(bytes));
            }

            hashes = next_level;
        }

        hashes[0]
    }
}
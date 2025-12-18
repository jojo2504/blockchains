use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::types::{subtypes::Hash256, transaction::Transaction};

#[derive(Debug, Serialize, Deserialize)]
pub struct MiningJob {
    pub parent_block_hash: Hash256,
    pub difficulty: BigUint,
    pub transactions: Vec<Transaction>,
    pub target_block_time: u64,
    pub height: u64,
}
use num_bigint::BigUint;

use crate::{blockchain::blockchain::Blockchain, consensus::difficulty::Difficulty, types::block::Block};

pub const TARGET_BLOCK_TIME: u64 = 4u64; // in seconds
const ADJUSTMENT_INTERVAL: usize = 10; // number of blocks between difficulty recalculation

#[derive(Clone)]
pub struct State {
    pub difficulty: Difficulty
}

impl State {
    pub fn adjust_difficulty(&mut self, slice: &[Block; 2]) {
        let actual_time = slice[1].timestamp.unwrap().signed_duration_since(slice[0].timestamp.unwrap());    
        let seconds = actual_time.num_seconds() as u64;
        self.difficulty.target *= BigUint::from(seconds / TARGET_BLOCK_TIME);
    }
}

#[derive(Clone)]
pub struct Consensus {
    pub state: State
}

impl Consensus {
    pub fn new(state: State) -> Self {
        Self { state }
    }

    fn push_to_blockchain(blockchain: &mut Blockchain, block: Block) {
        blockchain.push(block);
    }

    pub fn verify_valid_block(&self, last_block: &Block, block: &Block) -> bool {
        // verify hash difficulty
        if !self.state.difficulty.verify(block.hash) {
            return false;
        }

        // verify that the block hash is indeed the right one
        if block.calculate_hash() != block.hash {
            return false;
        }

        // verify block new pointer
        if block.previous_hash != Some(last_block.hash) {
            return false;
        }

        true
    }
}
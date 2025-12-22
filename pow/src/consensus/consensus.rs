use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{consensus::difficulty::Difficulty, types::block::Block};

pub const TARGET_BLOCK_TIME: u64 = 4u64; // in seconds
pub const ADJUSTMENT_INTERVAL: u64 = 5u64; // number of blocks between difficulty recalculation

#[derive(Clone)]
pub struct State {
    pub difficulty: Difficulty,
    pub start_window_time: DateTime<Utc>
}

impl State {
    pub fn adjust_difficulty(&mut self, stop_window_time: DateTime<Utc>) {
        let actual_time = (stop_window_time - self.start_window_time).num_seconds().max(1) as u64;  
        let mut new_target = &self.difficulty.target * BigUint::from(actual_time / (TARGET_BLOCK_TIME * ADJUSTMENT_INTERVAL));
        
        let max_up = &self.difficulty.target * BigUint::from(4u8);
        let max_down = &self.difficulty.target / BigUint::from(4u8);

        if new_target > max_up {
            new_target = max_up;
        } else if new_target < max_down {
            new_target = max_down;
        }

        self.difficulty.target = new_target;
        self.start_window_time = stop_window_time; 
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

    pub fn verify_valid_block(&self, last_block: &Block, block: &Block) -> bool {
        // verify hash difficulty
        if !self.state.difficulty.verify(block.hash) {
            return false;
        }

        // verify that the block hash is indeed the right one
        if block.calculate_hash() != block.hash {
            println!("block hash is bad ?");
            return false;
        }

        println!("block hash is good");

        // verify block new pointer
        if block.previous_hash != Some(last_block.hash) {
            return false;
        }

        true
    }
}
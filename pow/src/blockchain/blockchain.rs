use crate::{consensus::consensus::ADJUSTMENT_INTERVAL, types::block::Block};
use core::fmt;

/// The blockchain struct is a singleton
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Blockchain with {} blocks:", self.blocks.len())?;
        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, "Block {}:", i)?;
            writeln!(f, "{}", block)?;
        }
        Ok(())
    }
}

impl Blockchain {
    pub fn new(genesis: Block) -> Self {
        Self {
            blocks: vec![genesis],
        }
    }

    pub fn verify_integrity(&self) -> bool {
        // get first block -> the genesis
        let genesis = self.blocks.first().expect("Should always have at least the genesis block");
        if genesis.previous_hash.is_some() {
            println!("genesis has some previous hash");
            return false
        }
        
        let mut current_hash = genesis.calculate_hash();
        // println!("current hash {}", current_hash);
        
        for block in self.blocks.iter().clone().into_iter().skip(1) {
            if block.previous_hash.unwrap() != current_hash {
                println!("current block previous hash doesn't match last get hash");
                return false
            }
            current_hash = block.calculate_hash();
        }

        return true;
    }

    pub fn last_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }
}
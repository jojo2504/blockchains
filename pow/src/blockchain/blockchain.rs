use crate::types::block::Block;
use core::fmt;

/// The blockchain struct is a singleton
/// 
/// `difficulty` represents the number of zeroes we want at the beginning of each block's hash
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
    pub fn new() -> Self {
        let mut genesis = Block::new(vec![], None);
        genesis.hash = genesis.calculate_hash();
        Self {
            blocks: vec![genesis],
        }
    }

    /// Add a new block to the blockchain, emptying pending transactions by commiting all of them
    pub fn push(&mut self, block: Block) {
        // let pending = take(&mut self.pending_tx);
        
        // for tx in &pending {
        //     tx.commit();
        // }

        // self.blocks.push(block);
        todo!()
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
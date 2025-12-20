use std::collections::HashSet;

use crate::types::transaction::Transaction;

#[derive(Default, PartialEq, Eq)]
pub struct Mempool {
    pub pool: HashSet<Transaction>
}

impl Mempool {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn insert(&mut self, tx: Transaction) {
        self.pool.insert(tx);
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }
}
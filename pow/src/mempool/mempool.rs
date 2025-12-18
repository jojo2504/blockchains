use crate::types::transaction::Transaction;

#[derive(Default)]
pub struct Mempool {
    pub pool: Vec<Transaction>
}

impl Mempool {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn insert(&mut self, tx: Transaction) {
        self.pool.push(tx);
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }
}
use std::{collections::HashMap, marker::PhantomData};

use crate::{mempool::{self, mempool::Mempool}, types::transaction::Transaction};

struct NotConnected;
struct Connected;

pub struct Miner<State = NotConnected> {
    pool_snapshot:  HashMap<String, Transaction>,
    state: PhantomData<State>
}

impl Miner<NotConnected> {
    /// should connect to a node and 
    pub fn connect(&self) {
        todo!()
    }
}

impl Miner<Connected> {
    pub fn get_snapshot(&mut self, mempool: Mempool){
        todo!()
    }

    pub fn mining(&self) {
        loop {

        }
    }
}
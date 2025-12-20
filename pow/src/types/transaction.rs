use core::fmt;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::subtypes::{Address, Signature};

#[derive(Clone, Debug, Encode, Decode, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub txid: String,
    pub from: Address,
    pub to: Option<Address>,   // recipient, None if just a message
    pub nonce: u64,
    pub fee: u64,
    pub size: usize,
    pub amount: Option<u64>,   // None if just a message
    pub message: Option<Vec<u8>>,
    pub signature: Signature,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transaction {{\n  txid: {:?},\n  from: {:?},\n  to: {:?},\n  nonce: {},\n  fee: {},\n  size: {},\n  amount: {:?},\n  message: {:?},\n  signature: {:?}\n}}",
            self.txid,
            self.from,
            self.to,
            self.nonce,
            self.fee,
            self.size,
            self.amount,
            self.message,
            self.signature,
        )
    }
}

impl Transaction {
    pub fn new(
        txid: String,
        from: Address,
        to: Option<Address>,
        nonce: u64,
        fee: u64,
        amount: Option<u64>,
        message: Option<Vec<u8>>,
        signature: Signature,
    ) -> Self {
        Self {
            txid,
            from,
            to,
            nonce,
            fee,
            size: 0,
            amount,
            message,
            signature,
        }
    }

    pub fn commit(&self) {
        todo!()
    }
}
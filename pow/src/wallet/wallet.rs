use crate::types::subtypes::{Address, Seed};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wallet {
    pub seed: Seed,
    pub address: Address,
}

impl Wallet {
    pub fn new(seed: Seed) -> Self {
        let address = Address::from_seed(&seed);
        Wallet { seed, address }
    }
}
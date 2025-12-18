use num_bigint::BigUint;

use crate::types::subtypes::Hash256;

#[derive(Clone)]
pub struct Difficulty {
    pub target: BigUint
}

impl Difficulty {
    pub fn verify(&self, hash: Hash256) -> bool {
        let big_int = BigUint::from_bytes_be(&hash.0);
        big_int <= self.target
    }
}
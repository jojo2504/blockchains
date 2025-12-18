use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash256(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize)]
pub struct Seed (pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize)]
pub struct Address (pub [u8; 32]);

impl Address {
    pub fn from_seed(seed: &Seed) -> Self {
        // let hasher = Sha256::digest(seed);

        let hasher = Sha256::digest(seed.0);
        let address = hasher.as_chunks::<32>().0[0];

        Self(address)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize)]
pub struct Signature ([u8; 32]);




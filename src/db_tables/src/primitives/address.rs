use std::str::FromStr;

use secp256k1::{rand::rngs::OsRng, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::primitives::{
    base::ByteVec,
    crypto::{PublicKey, SecretKey},
    node::Error,
};

/// Represents a secp256k1 public key, hashed with sha256::digest
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Address(PublicKey);

impl Address {
    pub fn new(public_key: PublicKey) -> Self {
        Self(public_key)
    }

    pub fn public_key(&self) -> PublicKey {
        self.0
    }

    pub fn public_key_bytes(&self) -> ByteVec {
        // TODO: revisit later
        self.0.to_string().into_bytes()
    }
}

impl Default for Address {
    fn default() -> Self {
        // NOTE: should never panic as it's a valid string
        // TODO: impl default null public keys to avoid this call to expect
        let pk = PublicKey::from_str("null-address").expect("cant create null address");
        Self(pk)
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PublicKey::from_str(s)
            .map_err(|err| Error::Other(err.to_string()))
            .map(Self)
    }
}

impl From<String> for Address {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap_or_default()
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

pub type AccountKeypair = (secp256k1::SecretKey, secp256k1::PublicKey);

pub fn generate_account_keypair() -> AccountKeypair {
    let secp = Secp256k1::new();
    secp.generate_keypair(&mut OsRng)
}

pub fn generate_mock_account_keypair() -> AccountKeypair {
    type H = secp256k1::hashes::sha256::Hash;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_hashed_data::<H>(b"vrrb");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    (secret_key, public_key)
}

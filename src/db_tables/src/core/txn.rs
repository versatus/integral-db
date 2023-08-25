use crate::primitives::{
    address::Address,
    base::ByteVec,
    crypto::{PublicKey, Signature},
    digest::Digest as PrimitiveDigest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub type TxNonce = u128;
pub type TxTimestamp = i64;
pub type TxAmount = u128;

// TODO: replace with a generic token struct
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq)]
pub struct Txn {
    pub id: TransactionDigest,
    pub timestamp: TxTimestamp,
    pub sender_address: Address,
    pub sender_public_key: PublicKey,
    pub receiver_address: Address,
    pub token: Token,
    pub amount: TxAmount,
    pub signature: Signature,
    pub validators: Option<HashMap<String, bool>>,
    pub nonce: TxNonce,
}

impl Txn {
    pub fn timestamp(&self) -> TxTimestamp {
        self.timestamp
    }

    pub fn sender_address(&self) -> Address {
        self.sender_address.clone()
    }

    pub fn sender_public_key(&self) -> PublicKey {
        self.sender_public_key
    }

    pub fn receiver_address(&self) -> Address {
        self.receiver_address.clone()
    }

    pub fn token(&self) -> Token {
        self.token.clone()
    }

    pub fn amount(&self) -> TxAmount {
        self.amount
    }

    pub fn nonce(&self) -> TxNonce {
        self.nonce
    }

    pub fn generate_txn_digest_vec(&self) -> ByteVec {
        generate_txn_digest_vec(
            self.timestamp(),
            self.sender_address().to_string(),
            self.sender_public_key(),
            self.receiver_address().to_string(),
            self.token(),
            self.amount(),
            self.nonce(),
        )
    }
}

pub fn generate_txn_digest_vec(
    timestamp: TxTimestamp,
    sender_address: String,
    sender_public_key: PublicKey,
    receiver_address: String,
    token: Token,
    amount: TxAmount,
    nonce: TxNonce,
) -> ByteVec {
    let payload_string = format!(
        "{},{},{},{},{},{:?},{}",
        &timestamp, &sender_address, &sender_public_key, &receiver_address, &amount, &token, &nonce
    );

    let mut hasher = Sha256::new();
    hasher.update(payload_string);
    let hash = hasher.finalize();

    hash.to_vec()
}

impl PartialEq for Txn {
    fn eq(&self, other: &Self) -> bool {
        self.generate_txn_digest_vec() == other.generate_txn_digest_vec()
    }
}

#[derive(Debug, Default, Clone, Hash, Deserialize, Serialize, Eq, PartialEq)]
pub struct TransactionDigest {
    inner: PrimitiveDigest,
    digest_string: String,
}

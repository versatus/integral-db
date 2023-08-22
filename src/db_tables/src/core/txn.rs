use crate::primitives::{
    address::Address,
    crypto::{PublicKey, Signature},
    digest::Digest as PrimitiveDigest,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Default, Clone, Hash, Deserialize, Serialize, Eq, PartialEq)]
pub struct TransactionDigest {
    inner: PrimitiveDigest,
    digest_string: String,
}

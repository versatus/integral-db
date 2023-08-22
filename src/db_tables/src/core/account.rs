use crate::core::txn::TransactionDigest;
use crate::primitives::{address::Address, crypto::SerializedPublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Struct representing the desired updates to be applied to account.
/// TODO: impl Default for UpdateArgs { ... }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UpdateArgs {
    pub address: Address,
    pub nonce: Option<u128>,
    pub credits: Option<u128>,
    pub debits: Option<u128>,
    pub storage: Option<Option<String>>,
    pub code: Option<Option<String>>,
    pub digests: Option<AccountDigests>,
}

pub type AccountNonce = u128;

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Account {
    address: Address,
    hash: String,
    nonce: AccountNonce,
    credits: u128,
    debits: u128,
    storage: Option<String>,
    code: Option<String>,
    pubkey: SerializedPublicKey,
    digests: AccountDigests,
    created_at: i64,
    updated_at: Option<i64>,
}

/// Wrapper to provide convenient access to all the digests
/// throughout the history of a given account, separated by whether
/// the txn was sent from the account, received by the account, or
/// was a staking transaction.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AccountDigests {
    sent: HashSet<TransactionDigest>,
    recv: HashSet<TransactionDigest>,
    stake: HashSet<TransactionDigest>,
    //TODO: Add withdrawaltransaction digests for
    //withdrawing stake.
}

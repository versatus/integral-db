use crate::core::staking::Stake;
use crate::primitives::{address::Address, crypto::PublicKey};
use ethereum_types::U256;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// The claim object that stores the key information used to mine blocks,
/// calculate whether or not you are an entitled miner, and to share with
/// network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Claim {
    pub public_key: PublicKey,
    pub address: Address,
    pub hash: U256,
    pub eligibility: Eligibility,
    pub ip_address: SocketAddr,
    pub signature: String,
    stake: u128,
    stake_txns: Vec<Stake>,
}

///Node has privileges to be Miner/Validator,Farmer or None
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Eligibility {
    Harvester,
    Miner,
    Farmer,
    None,
}

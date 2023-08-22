use crate::core::MinerPk;
use crate::primitives::{
    address::Address,
    base::{PayloadHash, QuorumPublicKey},
    crypto::Signature,
};
use serde::{Deserialize, Serialize};

/// Represents a byte array that can be converted into a
/// ThresholdSignature
pub type Certificate = (Vec<u8>, PayloadHash);

/// A struct thatt defines a stake, includes the public key (which
/// can be converted into an address) an amount, which is an instance
/// of the `StakeUpdate` enum, a timestamp to sequence it in the
/// `StakeTxns` field of the claim, and a signature to verify it indeed
/// came from the publickey in question.
///
/// Also includes an optional address, if `None` is provided then
/// it is assumed the stake is directed to the claim address associated
/// with the pubkey field in this struct. If `Some` is provided then it
/// is assumed that the stake is being delegated to another node.
///
/// ```
/// use primitives::{Address, Signature};
/// use serde::{Deserialize, Serialize};
/// use vrrb_core::{
///     keypair::{MinerPk, MinerSk},
///     staking::StakeUpdate,
/// };
///
/// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// pub struct Stake {
///     pubkey: MinerPk,
///     from: Address,
///     to: Option<Address>,
///     amount: StakeUpdate,
///     timestamp: i64,
///     signature: Signature,
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Stake {
    pubkey: MinerPk,
    from: Address,
    to: Option<Address>,
    amount: StakeUpdate,
    timestamp: i64,
    signature: Signature,
    validator_quorum_key: QuorumPublicKey,
    certificate: Option<Certificate>,
}

/// Provides an enum with the 3 different types of StakeUpdates that
/// are possible, and a inner value which is the amount (for Add and
/// Withdrawal variants) and the percent to slash (for Slash) variant.
///
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// pub enum StakeUpdate {
///     Add(u128),
///     Withdrawal(u128),
///     Slash(u8),
/// }
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StakeUpdate {
    Add(u128),
    Withdrawal(u128),
    Slash(u8),
}

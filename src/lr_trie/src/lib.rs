/// This crate contains a left-right wrapped, evmap-backed JellyfishMerkleTree,
/// which is a modified version Penumbra & Diem's JellyfishMerkleTree to suite
/// our concurrent read and write needs.
pub use patriecia::H256;

mod absorb_op;
pub mod op;
mod result;
mod tree_wrapper;
mod trie;

pub use crate::{absorb_op::*, op::*, result::*, tree_wrapper::*, trie::*};

/// This crate contains a left-right wrapped, evmap-backed, Merkle-Patricia Trie
/// heavily inspired by https://github.com/carver/eth-trie.rs which is a fork of https://github.com/citahub/cita-trie
pub use patriecia::H256;

mod absorb_op;
pub mod op;
mod result;
mod tree_wrapper;
mod trie;

pub use crate::{absorb_op::*, op::*, result::*, tree_wrapper::*, trie::*};

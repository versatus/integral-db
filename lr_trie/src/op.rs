pub type Byte = u8;
pub type Bytes = [Byte];
pub type Key = Vec<Byte>;
pub type TrieValue = Vec<Byte>;
use patriecia::{KeyHash, OwnedValue, Version};

#[derive(Debug)]
pub enum Operation {
    /// Add a single value serialized to bytes at a specified version
    Add((KeyHash, Option<OwnedValue>), Version),
    /// Remove a value specified by the key and version from the trie
    Remove(KeyHash, Version),
}

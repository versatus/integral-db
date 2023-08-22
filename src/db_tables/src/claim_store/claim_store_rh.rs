use std::collections::HashMap;

use crate::core::claim::Claim;
use crate::primitives::node::NodeId;
use crate::storage_utils::result::{Result, StorageError};
use lr_trie::{JellyfishMerkleTreeWrapper, ReadHandleFactory};
use patriecia::JellyfishMerkleTree;

use crate::RocksDbAdapter;

#[derive(Debug, Clone)]
pub struct ClaimStoreReadHandle {
    inner: JellyfishMerkleTreeWrapper<RocksDbAdapter>,
}

impl ClaimStoreReadHandle {
    pub fn new(inner: JellyfishMerkleTreeWrapper<RocksDbAdapter>) -> Self {
        Self { inner }
    }

    /// Returns `Some(Claim)` if an account exist under given PublicKey.
    /// Otherwise returns `None`.
    pub fn get(&self, key: &NodeId) -> Result<Claim> {
        self.inner
            .get(key)
            .map_err(|err| StorageError::Other(err.to_string()))
    }

    /// Get a batch of claims by providing Vec of PublicKeysHash
    ///
    /// Returns HashMap indexed by PublicKeys and containing either
    /// Some(account) or None if account was not found.
    pub fn batch_get(&self, keys: Vec<NodeId>) -> HashMap<NodeId, Option<Claim>> {
        let mut claims = HashMap::new();

        keys.iter().for_each(|key| {
            let value = self.get(key).ok();
            claims.insert(key.to_owned(), value);
        });

        claims
    }

    pub fn entries(&self) -> HashMap<NodeId, Claim> {
        // TODO: revisit and refactor into inner wrapper
        self.inner
            .iter()
            .filter_map(|(key, value)| {
                if let Ok(key) = bincode::deserialize(&key) {
                    if let Ok(value) = bincode::deserialize(&value) {
                        return Some((key, value));
                    }
                }
                None
            })
            .collect()
    }

    /// Returns a number of initialized claims in the database
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the information about the ClaimDb being empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ClaimStoreReadHandleFactory {
    inner: ReadHandleFactory<JellyfishMerkleTree<RocksDbAdapter>>,
}

impl ClaimStoreReadHandleFactory {
    pub fn new(inner: ReadHandleFactory<JellyfishMerkleTree<RocksDbAdapter>>) -> Self {
        Self { inner }
    }

    pub fn handle(&self) -> ClaimStoreReadHandle {
        let handle = self
            .inner
            .handle()
            .enter()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        let inner = JellyfishMerkleTreeWrapper::new(handle);

        ClaimStoreReadHandle { inner }
    }
}

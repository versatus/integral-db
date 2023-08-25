use std::collections::HashMap;

use crate::core::account::Account;
use crate::primitives::address::Address;
use crate::storage_utils::result::{Result, StorageError};
use lr_trie::{JellyfishMerkleTreeWrapper, ReadHandleFactory};
use patriecia::{JellyfishMerkleTree, SimpleHasher};

use crate::rocksdb_adapter::RocksDbAdapter;

#[derive(Debug, Clone)]
pub struct StateStoreReadHandle<H: SimpleHasher> {
    inner: JellyfishMerkleTreeWrapper<RocksDbAdapter, H>,
}

impl<H: SimpleHasher> StateStoreReadHandle<H> {
    pub fn new(inner: JellyfishMerkleTreeWrapper<RocksDbAdapter, H>) -> Self {
        Self { inner }
    }

    /// Returns `Some(Account)` if an account exist under given PublicKey.
    /// Otherwise returns `None`.
    pub fn get(&self, key: &Address) -> Result<Account> {
        self.inner
            .get(key)
            .map_err(|err| StorageError::Other(err.to_string()))
    }

    /// Get a batch of accounts by providing Vec of PublicKeysHash
    ///
    /// Returns HashMap indexed by PublicKeys and containing either
    /// Some(account) or None if account was not found.
    pub fn batch_get(&self, keys: Vec<Address>) -> HashMap<Address, Option<Account>> {
        let mut accounts = HashMap::new();

        keys.iter().for_each(|key| {
            let value = self.get(key).ok();
            accounts.insert(key.to_owned(), value);
        });

        accounts
    }

    pub fn entries(&self) -> HashMap<Address, Account> {
        // TODO: revisit and refactor into inner wrapper
        self.inner
            .iter()
            .filter_map(|(key, value)| {
                if let Ok(key) = bincode::deserialize(&key) {
                    let value = bincode::deserialize(&value).unwrap_or_default();

                    return Some((key, value));
                }
                None
            })
            .collect()
    }

    /// Returns a number of initialized accounts in the database
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the information about the StateDb being empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct StateStoreReadHandleFactory<H: SimpleHasher> {
    inner: ReadHandleFactory<JellyfishMerkleTree<RocksDbAdapter, H>>,
}

impl<H: SimpleHasher> StateStoreReadHandleFactory<H> {
    pub fn new(inner: ReadHandleFactory<JellyfishMerkleTree<RocksDbAdapter, H>>) -> Self {
        Self { inner }
    }

    pub fn handle(&self) -> StateStoreReadHandle<H> {
        let handle = self
            .inner
            .handle()
            .enter()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        let inner = JellyfishMerkleTreeWrapper::new(handle);

        StateStoreReadHandle { inner }
    }
}

use std::collections::HashMap;

use crate::core::txn::{TransactionDigest, Txn};
use crate::storage_utils::result::{Result, StorageError};
use lr_trie::{JellyfishMerkleTreeWrapper, ReadHandleFactory};
use patriecia::{JellyfishMerkleTree, SimpleHasher};

use crate::rocksdb_adapter::RocksDbAdapter;

#[derive(Debug, Clone)]
pub struct TransactionStoreReadHandle<H: SimpleHasher> {
    inner: JellyfishMerkleTreeWrapper<RocksDbAdapter, H>,
}

impl<H: SimpleHasher> TransactionStoreReadHandle<H> {
    pub fn new(inner: JellyfishMerkleTreeWrapper<RocksDbAdapter, H>) -> Self {
        Self { inner }
    }

    pub fn get(&self, key: &TransactionDigest) -> Result<Txn> {
        self.inner
            .get(key)
            .map_err(|err| StorageError::Other(err.to_string()))
    }

    pub fn batch_get(
        &self,
        keys: Vec<TransactionDigest>,
    ) -> HashMap<TransactionDigest, Option<Txn>> {
        let mut transactions = HashMap::new();

        keys.iter().for_each(|key| {
            let value = self.get(key).ok();
            transactions.insert(key.to_owned(), value);
        });

        transactions
    }

    pub fn entries(&self) -> HashMap<TransactionDigest, Txn> {
        // TODO: revisit and refactor into inner wrapper
        self.inner
            .iter()
            .map(|(key, value)| {
                let key = bincode::deserialize(&key).unwrap_or_default();
                let value = bincode::deserialize(&value).unwrap_or_default();

                (key, value)
            })
            .collect()
    }

    /// Returns a number of transactions in the ledger
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if TransactionStore is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionStoreReadHandleFactory<H: SimpleHasher> {
    inner: ReadHandleFactory<JellyfishMerkleTree<RocksDbAdapter, H>>,
}

impl<H: SimpleHasher> TransactionStoreReadHandleFactory<H> {
    pub fn new(inner: ReadHandleFactory<JellyfishMerkleTree<RocksDbAdapter, H>>) -> Self {
        Self { inner }
    }

    pub fn handle(&self) -> TransactionStoreReadHandle<H> {
        let handle = self
            .inner
            .handle()
            .enter()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        let inner = JellyfishMerkleTreeWrapper::new(handle);

        TransactionStoreReadHandle { inner }
    }
}

use std::{path::Path, sync::Arc};

use crate::core::txn::{TransactionDigest, Txn};
use crate::storage_utils::{self, result::Result};
use lr_trie::{LeftRightTrie, Proof, H256};
use patriecia::SimpleHasher;

use crate::rocksdb_adapter::RocksDbAdapter;

mod transaction_store_rh;
pub use transaction_store_rh::*;

#[derive(Debug, Clone)]
pub struct TransactionStore<H: SimpleHasher> {
    trie: LeftRightTrie<'static, TransactionDigest, Txn, RocksDbAdapter, H>,
}

impl<H: SimpleHasher> Default for TransactionStore<H> {
    fn default() -> Self {
        let db_path = storage_utils::get_node_data_dir()
            .unwrap_or_default()
            .join("db")
            .join("transactions");

        let db_adapter = RocksDbAdapter::new(db_path, "transactions").unwrap_or_default();

        let trie = LeftRightTrie::new(Arc::new(db_adapter));

        Self { trie }
    }
}

impl<H: SimpleHasher> TransactionStore<H> {
    /// Returns new, empty instance of TransactionStore
    pub fn new(path: &Path) -> Self {
        let path = path.join("transactions");
        let db_adapter = RocksDbAdapter::new(path, "transactions").unwrap_or_default();
        let trie = LeftRightTrie::new(Arc::new(db_adapter));

        Self { trie }
    }

    pub fn factory(&self) -> TransactionStoreReadHandleFactory<H> {
        let inner = self.trie.factory();

        TransactionStoreReadHandleFactory::new(inner)
    }

    pub fn commit(&mut self) {
        self.trie.publish();
    }

    pub fn read_handle(&self) -> TransactionStoreReadHandle<H> {
        let inner = self.trie.handle().unwrap();
        TransactionStoreReadHandle::new(inner)
    }

    pub fn insert(&mut self, txn: Txn) -> Result<()> {
        self.trie.insert(txn.digest(), txn);
        Ok(())
    }

    pub fn extend(&mut self, transactions: Vec<Txn>) {
        let transactions = transactions
            .into_iter()
            .map(|txn| (txn.digest(), txn))
            .collect();

        self.trie.extend(transactions)
    }

    pub fn root_hash(&self) -> Option<H256> {
        self.trie.root()
    }

    pub fn get_proof(&self) -> Result<Vec<Proof>> {
        todo!()
    }

    pub fn verify_proof(&self) -> Option<H256> {
        todo!()
    }
}

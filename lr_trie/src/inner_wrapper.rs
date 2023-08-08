use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

pub use left_right::ReadHandleFactory;
use patriecia::{
    JellyfishMerkleIterator, JellyfishMerkleTree, SimpleHasher, TreeReader, VersionedDatabase, H256,
};
use serde::{Deserialize, Serialize};

use crate::{LeftRightTrieError, Result};

pub type Proof = Vec<u8>;

#[derive(Debug, Clone)]
pub struct JMTWrapper<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    inner: JellyfishMerkleTree<'a, D, H>,
}

impl<'a, D, H> JMTWrapper<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    pub fn new(inner: JellyfishMerkleTree<D, H>) -> Self {
        Self { inner }
    }

    /// Produces a clone of the underlying trie
    pub fn inner(&self) -> JellyfishMerkleTree<D, H> {
        self.inner.clone()
    }

    pub fn get<K, V>(&self, key: &K) -> Result<V>
    where
        K: for<'b> Deserialize<'b> + Serialize + Clone,
        V: for<'b> Deserialize<'b> + Serialize + Clone,
    {
        let key = bincode::serialize(key).unwrap_or_default();

        let raw_value_opt = self
            .inner
            .get(&key)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))?;

        let raw_value = raw_value_opt.ok_or_else(|| {
            LeftRightTrieError::Other("received none value from inner trie".to_string())
        })?;

        let value = bincode::deserialize::<V>(&raw_value)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))?;

        Ok(value)
    }

    pub fn contains<'b, K, V>(&self, key: &'a K) -> Result<bool>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = bincode::serialize(&key).unwrap_or_default();
        self.inner
            .contains(&key)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn insert<'b, K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = bincode::serialize(&key).unwrap_or_default();
        let value = bincode::serialize(&value).unwrap_or_default();

        self.inner
            .insert(&key, &value)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn remove<'b, K, V>(&mut self, key: K) -> Result<bool>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = bincode::serialize(&key).unwrap_or_default();
        self.inner
            .remove(&key)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn root_hash(&mut self) -> Result<H256> {
        self.commit()
    }

    /// Creates a Merkle proof for a given value.
    pub fn get_proof<'b, K, V>(&mut self, key: &K) -> Result<Vec<Proof>>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = bincode::serialize(key).unwrap_or_default();
        self.inner
            .get_proof(&key)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Verifies a Merkle proof for a given value.
    pub fn verify_proof<'b, K, V>(
        &self,
        root_hash: H256,
        key: &K,
        proof: Vec<Proof>,
    ) -> Result<Option<Proof>>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = bincode::serialize(key).unwrap_or_default();

        self.inner
            .verify_proof(root_hash, &key, proof)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn commit(&mut self) -> Result<H256> {
        self.inner
            .commit()
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn iter(&self) -> JellyfishMerkleIterator<D> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn db(&self) -> Arc<D> {
        self.inner.db()
    }
}

impl<'a, D, H> Display for JMTWrapper<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

use std::fmt::{self, Debug, Display, Formatter};

pub use left_right::ReadHandleFactory;
use patriecia::{
    JellyfishMerkleIterator, JellyfishMerkleTree, KeyHash, RootHash, Sha256, SimpleHasher,
    SparseMerkleProof, TreeReader, TreeWriter, Version, VersionedDatabase, VersionedTrie,
};
use serde::{Deserialize, Serialize};

use crate::{LeftRightTrieError, Result};

pub type Proof = Vec<u8>;

#[derive(Debug, Clone)]
pub struct JellyfishMerkleTreeWrapper<D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    inner: JellyfishMerkleTree<D, H>,
}

impl<D, H> JellyfishMerkleTreeWrapper<D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    pub fn new(inner: JellyfishMerkleTree<D, H>) -> Self {
        Self { inner }
    }

    /// Produces a clone of the underlying trie
    pub fn inner(&self) -> JellyfishMerkleTree<D, H> {
        self.inner.clone()
    }

    /// Get the value associated with a key at a specified `Version`.
    pub fn get<K, V>(&self, key: &K, version: Version) -> Result<V>
    where
        K: for<'b> Deserialize<'b> + Serialize + Clone,
        V: for<'b> Deserialize<'b> + Serialize + Clone,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());

        let raw_value_opt = self
            .inner
            .get(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))?;

        let raw_value = raw_value_opt.ok_or_else(|| {
            LeftRightTrieError::Other("received none value from inner trie".to_string())
        })?;

        let value = bincode::deserialize::<V>(&raw_value)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))?;

        Ok(value)
    }

    /// Returns true if the inner tree contains the specified key at `Version`.
    pub fn contains<'b, K>(&self, key: &'b K, version: Version) -> Result<bool>
    where
        K: Serialize + Deserialize<'b>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        self.inner
            .contains(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Insert a key-value pair into the tree at a specified `Version` and update the database
    /// from the node batch produced.
    pub fn insert<'b, K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: Serialize + Deserialize<'b>,
        V: Serialize + Deserialize<'b>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        let value = bincode::serialize(&value).unwrap_or_default();

        match self
            .inner
            .put_value_set(vec![(key, Some(value))], self.version() + 1)
        {
            Ok((_, batch)) => self
                .inner
                .reader()
                .write_node_batch(&batch.node_batch)
                .map_err(|err| LeftRightTrieError::Other(err.to_string())),
            Err(err) => Err(LeftRightTrieError::Other(err.to_string())),
        }
    }

    /// Given a key, remove a value from the tree at a specified `Version` and update the database
    /// from the node batch produced.
    ///
    /// Returns true if the value for key at version is no longer contained within the tree.
    pub fn remove<'b, K>(&mut self, key: K) -> Result<bool>
    where
        K: Serialize + Deserialize<'b>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        let version = self.version() + 1;
        match self.inner.put_value_set(vec![(key, None)], version) {
            Ok((_, batch)) => self
                .inner
                .reader()
                .write_node_batch(&batch.node_batch)
                .map_err(|err| LeftRightTrieError::Other(err.to_string()))?,
            Err(err) => return Err(LeftRightTrieError::Other(err.to_string())),
        }
        Ok(!self
            .inner
            .contains(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))?)
    }

    /// Get the `RootHash` of a `JellyfishMerkleTree` at a specified `Version`.
    pub fn root_hash(&self, version: Version) -> Result<RootHash> {
        self.inner
            .get_root_hash(version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Creates a Merkle proof for a given value.
    pub fn get_proof<'b, K>(&mut self, key: &K, version: Version) -> Result<SparseMerkleProof<H>>
    where
        K: Serialize + Deserialize<'b>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        self.inner
            .get_proof(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Verifies a Merkle proof for a given value.
    pub fn verify_proof<'b, K>(
        &self,
        element_key: KeyHash,
        version: Version,
        expected_root_hash: RootHash,
        proof: SparseMerkleProof<H>,
    ) -> Result<()>
    where
        K: Serialize + Deserialize<'b>,
    {
        self.inner
            .verify_proof(element_key, version, expected_root_hash, proof)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Create a [`JellyfishMerkleIterator`] from the reader: R, to iterate
    /// over values in the tree starting at the given key and version.
    pub fn iter(
        &self,
        version: Version,
        starting_key: KeyHash,
    ) -> Result<JellyfishMerkleIterator<D>> {
        self.inner
            .iter(version, starting_key)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Get the number of `Some(value)`s from the latest version of the tree stored in the `VersionedDatabase`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if there are no nodes with `OwnedValue`s for the latest
    /// `Version` in `VersionedDatabase::value_history()`
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the latest `Version` of the tree known to the database.
    pub fn version(&self) -> Version {
        self.inner.version()
    }

    /// Returns a clone of the value history from the database.
    pub fn value_history(&self) -> <D as VersionedDatabase>::HistoryIter {
        self.inner.reader().value_history()
    }
}

impl<D, H> Display for JellyfishMerkleTreeWrapper<D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use patriecia::MockTreeStore;

    use super::*;

    #[test]
    fn test_wrapper_can_add_remove_values() {
        let db = Arc::new(MockTreeStore::default());
        let jmt = JellyfishMerkleTree::<_, Sha256>::new(db);
        let mut wrapper = JellyfishMerkleTreeWrapper::new(jmt);

        let key = "Ada Lovelace";
        let value = "Analytical Engine";
        let mut version = 1;

        wrapper.insert(key, value).unwrap();
        let contains_key = wrapper.contains(&key, version).unwrap();
        assert!(contains_key);

        version += 1; // update version when adding or removing
        wrapper.remove(key).unwrap();
        let contains_key = wrapper.contains(&key, version).unwrap();
        assert!(!contains_key);

        assert_eq!(
            wrapper.version(),
            2 /* there are two total transactions */
        );
    }
}

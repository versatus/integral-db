use std::fmt::{self, Debug, Display, Formatter};

pub use left_right::ReadHandleFactory;
use patriecia::{
    JellyfishMerkleIterator, JellyfishMerkleTree, KeyHash, RootHash, Sha256, SimpleHasher,
    SparseMerkleProof, TreeReader, Version, VersionedDatabase, VersionedTrie,
};
use serde::{Deserialize, Serialize};

use crate::{LeftRightTrieError, Result};

pub type Proof = Vec<u8>;

#[derive(Debug, Clone)]
pub struct JellyfishMerkleTreeWrapper<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    inner: JellyfishMerkleTree<'a, D, H>,
}

impl<'a, D, H> JellyfishMerkleTreeWrapper<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    pub fn new(inner: JellyfishMerkleTree<'a, D, H>) -> Self {
        Self { inner }
    }

    /// Produces a clone of the underlying trie
    pub fn inner(&self) -> JellyfishMerkleTree<D, H> {
        self.inner.clone()
    }

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

    pub fn contains<'b, K, V>(&self, key: &'a K, version: Version) -> Result<bool>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        self.inner
            .contains(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn insert<'b, K, V>(&mut self, key: K, value: V, version: Version) -> Result<()>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        let value = bincode::serialize(&value).unwrap_or_default();

        self.inner.put_value_set(vec![(key, Some(value))], version);
        Ok(())
    }

    /// Returns true if the value for key at version is not contained within the tree
    pub fn remove<'b, K, V>(&mut self, key: K, version: Version) -> Result<bool>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        self.inner.put_value_set(vec![(key, None)], version);
        Ok(!self
            .inner
            .contains(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))?)
    }

    pub fn root_hash(&self, version: Version) -> Result<RootHash> {
        self.inner
            .get_root_hash(version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Creates a Merkle proof for a given value.
    pub fn get_proof<'b, K, V>(&mut self, key: &K, version: Version) -> Result<SparseMerkleProof<H>>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        let key = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        self.inner
            .get_proof(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    /// Verifies a Merkle proof for a given value.
    pub fn verify_proof<'b, K, V>(
        &self,
        element_key: KeyHash,
        version: Version,
        expected_root_hash: RootHash,
        proof: SparseMerkleProof<H>,
    ) -> Result<()>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
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

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn version(&self) -> Version {
        self.inner.version()
    }
}

impl<'a, D, H> Display for JellyfishMerkleTreeWrapper<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

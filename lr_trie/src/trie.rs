use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    sync::Arc,
};

pub use left_right::ReadHandleFactory;
use left_right::{ReadHandle, WriteHandle};
use patriecia::{
    JellyfishMerkleTree, KeyHash, RootHash, Sha256, SimpleHasher, SparseMerkleProof, TreeReader,
    TreeWriter, Version, VersionedDatabase,
};
use serde::{Deserialize, Serialize};

use crate::{JellyfishMerkleTreeWrapper, LeftRightTrieError, Operation, Result};

/// Concurrent generic Merkle Patricia Trie
#[derive(Debug)]
pub struct LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    pub read_handle: ReadHandle<JellyfishMerkleTree<D, H>>,
    pub write_handle: WriteHandle<JellyfishMerkleTree<D, H>, Operation>,
    _marker: PhantomData<(K, V, &'a ())>,
}

impl<'a, D, K, V, H> LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    pub fn new(db: Arc<D>) -> Self {
        let (write_handle, read_handle) = left_right::new_from_empty(JellyfishMerkleTree::new(db));

        Self {
            read_handle,
            write_handle,
            _marker: PhantomData,
        }
    }

    pub fn handle(&self) -> Result<JellyfishMerkleTreeWrapper<D, H>> {
        if let Some(read_handle) = self.read_handle.enter().map(|guard| guard.clone()) {
            Ok(JellyfishMerkleTreeWrapper::new(read_handle))
        } else {
            Err(LeftRightTrieError::Other(
                "failed to retrieve read handle".to_string(),
            ))
        }
    }

    /// Returns a vector of all entries within the trie
    pub fn entries(&self) -> Vec<(K, V)> {
        todo!()
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.handle()?.len())
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.handle()?.is_empty())
    }

    pub fn root(&self, version: Version) -> Result<RootHash> {
        self.handle()?.root_hash(version)
    }

    pub fn version(&self) -> Result<Version> {
        Ok(self.handle()?.version())
    }

    /// Get the `RootHash` at the latest `LatestVersion`.
    pub fn root_latest(&self) -> Result<RootHash> {
        self.root(self.version()?)
    }

    pub fn get_proof(&'a mut self, key: &K, version: Version) -> Result<SparseMerkleProof<H>>
    where
        K: Serialize + Deserialize<'a>,
    {
        self.handle()?
            .get_proof::<K>(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn verify_proof(
        &'a self,
        element_key: KeyHash,
        version: Version,
        expected_root_hash: RootHash,
        proof: SparseMerkleProof<H>,
    ) -> Result<()>
    where
        K: Serialize + Deserialize<'a>,
    {
        self.handle()?
            .verify_proof::<K>(element_key, version, expected_root_hash, proof)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn factory(&'a self) -> ReadHandleFactory<JellyfishMerkleTree<D, H>> {
        self.read_handle.factory()
    }

    pub fn update(&mut self, key: K, value: V, version: Version) {
        self.insert(key, value, version)
    }

    pub fn publish(&mut self) {
        self.write_handle.publish();
    }

    pub fn insert(&mut self, key: K, value: V, version: Version) {
        //TODO: revisit the serializer used to store things on the trie
        let keyhash = KeyHash::with::<Sha256>(bincode::serialize(&key).unwrap_or_default());
        let owned_value = bincode::serialize(&value).unwrap_or_default();
        self.write_handle
            .append(Operation::Add((keyhash, Some(owned_value)), version))
            .publish();
    }

    // pub fn extend(&mut self, values: Vec<(K, V)>) {
    //     let mapped = values
    //         .into_iter()
    //         .map(|(key, value)| {
    //             //TODO: revisit the serializer used to store things on the trie
    //             let key = bincode::serialize(&key).unwrap_or_default();
    //             let value = bincode::serialize(&value).unwrap_or_default();

    //             (key, value)
    //         })
    //         .collect();

    //     self.write_handle
    //         .append(Operation::Extend(mapped))
    //         .publish();
    // }
}

impl<'a, D, K, V, H> PartialEq for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn eq(&self, other: &Self) -> bool {
        self.root_latest() == other.root_latest()
    }
}

impl<'a, D, K, V, H> From<Arc<D>> for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn from(db: Arc<D>) -> Self {
        let (write_handle, read_handle) = left_right::new_from_empty(JellyfishMerkleTree::new(db));

        Self {
            read_handle,
            write_handle,
            _marker: PhantomData,
        }
    }
}

impl<'a, D, K, V, H> From<JellyfishMerkleTree<D, H>> for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn from(other: JellyfishMerkleTree<D, H>) -> Self {
        let (write_handle, read_handle) = left_right::new_from_empty(other);

        Self {
            read_handle,
            write_handle,
            _marker: PhantomData,
        }
    }
}

/// Substitue clone trait.
/// Avoids creating temporary values which are freed while in use by
/// providing the method with a valid clone of a `JellyfishMerkleTree`.
///
/// # Example:
/// ```rust, ignore
/// let db = Default::default();
/// let lr_trie = LeftRightTree::new(&db);
///
/// let clone = LeftRightTree::from_clone(lr_trie.handle().inner());
/// assert_eq!(lr_trie, clone);
/// ```
trait SubClone<'a, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    fn from_clone(inner: JellyfishMerkleTree<D, H>) -> Self;
}

impl<'a, D, K, V, H> SubClone<'a, D, H> for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn from_clone(inner: JellyfishMerkleTree<D, H>) -> Self {
        LeftRightTrie::from(inner)
    }
}

impl<'a, D, K, V, H> Display for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.handle()
                .expect("failed to retrieve handle for formatter")
        )
    }
}

#[cfg(test)]
mod tests {
    use patriecia::MockTreeStore;
    use std::thread;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct CustomValue {
        pub data: usize,
    }

    #[test]
    fn should_store_arbitrary_values() {
        let db = Arc::new(MockTreeStore::new(true));
        let mut trie = LeftRightTrie::<_, _, _, Sha256>::new(db);

        trie.insert("abcdefg", CustomValue { data: 100 }, 0);

        let value: CustomValue = trie
            .handle()
            .unwrap()
            .get(&String::from("abcdefg"), 0)
            .unwrap();

        assert_eq!(value, CustomValue { data: 100 });
    }

    #[ignore = "currently does not compile due to lifetimes"]
    #[test]
    fn should_be_read_concurrently() {
        let db = Arc::new(MockTreeStore::new(true));
        let mut trie = LeftRightTrie::<_, _, _, Sha256>::new(db);

        let total = 18;

        for n in 0..total {
            let key = format!("test-{n}");

            trie.insert(key, CustomValue { data: 12345 }, n);
        }

        trie.publish();

        // NOTE Spawn 10 threads and 10 readers that should report the exact same value
        [0..10]
            .iter()
            .map(|_| {
                let reader = trie.handle().unwrap();
                thread::spawn(move || {
                    assert_eq!(reader.len() as u64, total);
                    for n in 0..total {
                        let key = format!("test-{n}");

                        let res: CustomValue = reader.get(&key, n).unwrap();

                        assert_eq!(res, CustomValue { data: 12345 });
                    }
                })
            })
            .for_each(|handle| {
                handle.join().unwrap();
            });
    }
}

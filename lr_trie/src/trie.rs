use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

pub use left_right::ReadHandleFactory;
use left_right::{ReadHandle, WriteHandle};
use patriecia::{
    JellyfishMerkleTree, KeyHash, RootHash, Sha256, SimpleHasher, SparseMerkleProof, TreeReader,
    Version, VersionedDatabase,
};
use serde::{Deserialize, Serialize};

use crate::{JellyfishMerkleTreeWrapper, LeftRightTrieError, Operation, Result};

/// Concurrent generic Merkle Patricia Trie
#[derive(Debug)]
pub struct LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    pub read_handle: ReadHandle<JellyfishMerkleTree<'a, D, H>>,
    pub write_handle: WriteHandle<JellyfishMerkleTree<'a, D, H>, Operation>,
    _marker: PhantomData<(K, V, &'a ())>,
}

impl<'a, D, K, V, H> LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    pub fn new(db: D) -> Self {
        let (write_handle, read_handle) = left_right::new_from_empty(JellyfishMerkleTree::new(&db));

        Self {
            read_handle,
            write_handle,
            _marker: PhantomData,
        }
    }

    pub fn handle(&self) -> JellyfishMerkleTreeWrapper<D, H> {
        let read_handle = self
            .read_handle
            .enter()
            .map(|guard| guard.clone())
            .unwrap_or({
                let db = D::default();
                JellyfishMerkleTree::new(&db)
            });

        JellyfishMerkleTreeWrapper::new(read_handle)
    }

    /// Returns a vector of all entries within the trie
    pub fn entries(&self) -> Vec<(K, V)> {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.handle().len()
    }

    pub fn is_empty(&self) -> bool {
        self.handle().is_empty()
    }

    pub fn root_opt(&self, version: Version) -> Option<RootHash> {
        self.handle().root_hash(version).ok()
    }

    pub fn version(&self) -> Version {
        self.handle().version()
    }

    pub fn get_proof(&mut self, key: &K, version: Version) -> Result<SparseMerkleProof<H>>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        self.handle()
            .get_proof::<K, V>(key, version)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn verify_proof(
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
        self.handle()
            .verify_proof::<K, V>(element_key, version, expected_root_hash, proof)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn factory(&self) -> ReadHandleFactory<JellyfishMerkleTree<D, H>> {
        self.read_handle.factory()
    }

    pub fn update(&mut self, key: K, value: V, version: Version) {
        self.insert(key, value, version);
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
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn eq(&self, other: &Self) -> bool {
        self.root_opt(self.version()) == other.root_opt(other.version())
    }
}

impl<'a, D, K, V, H> Default for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn default() -> Self {
        let db = D::default();
        let jmt = JellyfishMerkleTree::new(&db);
        let (write_handle, read_handle) =
            left_right::new_from_empty::<JellyfishMerkleTree<D, H>, Operation>(jmt);
        Self {
            read_handle,
            write_handle,
            _marker: PhantomData,
        }
    }
}

impl<'a, D, K, V, H> From<D> for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn from(db: D) -> Self {
        let (write_handle, read_handle) = left_right::new_from_empty(JellyfishMerkleTree::new(&db));

        Self {
            read_handle,
            write_handle,
            _marker: PhantomData,
        }
    }
}

impl<'a, D, K, V, H> From<JellyfishMerkleTree<'a, D, H>> for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
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

impl<'a, D, K, V, H> Clone for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn clone(&self) -> Self {
        let inner = self.handle().inner();
        LeftRightTrie::from(inner)
    }
}

impl<'a, D, K, V, H> Display for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.handle())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use patriecia::{MockTreeStore, VersionedDatabase};

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct CustomValue {
        pub data: usize,
    }

    #[test]
    fn should_store_arbitrary_values() {
        let memdb = MockTreeStore::new(true);
        let mut trie = LeftRightTrie::<_, _, _, Sha256>::new(memdb);

        trie.insert("abcdefg", CustomValue { data: 100 }, 0);

        let value: CustomValue = trie.handle().get(&String::from("abcdefg"), 0).unwrap();

        assert_eq!(value, CustomValue { data: 100 });
    }

    #[test]
    fn should_be_read_concurrently() {
        let db = MockTreeStore::new(true);
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
                let reader = trie.handle();
                thread::spawn(move || {
                    assert_eq!(db.len() as u64, total);
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

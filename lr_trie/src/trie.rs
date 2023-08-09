use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    sync::Arc,
};

pub use left_right::ReadHandleFactory;
use left_right::{ReadHandle, WriteHandle};
use patriecia::{JellyfishMerkleTree, SimpleHasher, TreeReader, VersionedDatabase, H256};
use serde::{Deserialize, Serialize};

use crate::{JMTWrapper, LeftRightTrieError, Operation, Proof, Result};

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

    pub fn handle(&self) -> JMTWrapper<D, H> {
        let read_handle = self
            .read_handle
            .enter()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        JMTWrapper::new(read_handle)
    }

    /// Returns a vector of all entries within the trie
    pub fn entries(&self) -> Vec<(K, V)> {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.handle().iter().count()
    }

    pub fn is_empty(&self) -> bool {
        self.handle().len() == 0
    }

    pub fn root(&self) -> Option<H256> {
        self.handle().root_hash().ok()
    }

    pub fn get_proof(&mut self, key: &K) -> Result<Vec<Proof>>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        self.handle()
            .get_proof::<K, V>(key)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn verify_proof(&self, root: H256, key: &K, proof: Vec<Proof>) -> Result<Option<Proof>>
    where
        K: Serialize + Deserialize<'a>,
        V: Serialize + Deserialize<'a>,
    {
        self.handle()
            .verify_proof::<K, V>(root, key, proof)
            .map_err(|err| LeftRightTrieError::Other(err.to_string()))
    }

    pub fn factory(&self) -> ReadHandleFactory<JellyfishMerkleTree<D, H>> {
        self.read_handle.factory()
    }

    pub fn update(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    pub fn publish(&mut self) {
        self.write_handle.publish();
    }

    pub fn insert(&mut self, key: K, value: V) {
        //TODO: revisit the serializer used to store things on the trie
        let key = bincode::serialize(&key).unwrap_or_default();
        let value = bincode::serialize(&value).unwrap_or_default();
        self.write_handle
            .append(Operation::Add(key, value))
            .publish();
    }

    pub fn extend(&mut self, values: Vec<(K, V)>) {
        let mapped = values
            .into_iter()
            .map(|(key, value)| {
                //TODO: revisit the serializer used to store things on the trie
                let key = bincode::serialize(&key).unwrap_or_default();
                let value = bincode::serialize(&value).unwrap_or_default();

                (key, value)
            })
            .collect();

        self.write_handle
            .append(Operation::Extend(mapped))
            .publish();
    }
}

impl<'a, D, K, V, H> PartialEq for LeftRightTrie<'a, K, V, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
    K: Serialize + Deserialize<'a>,
    V: Serialize + Deserialize<'a>,
{
    fn eq(&self, other: &Self) -> bool {
        self.handle().root_hash() == other.handle().root_hash()
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
        let (write_handle, read_handle) = left_right::new::<JellyfishMerkleTree<D, H>, Operation>();
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

    use patriecia::MockTreeStore;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct CustomValue {
        pub data: usize,
    }

    #[test]
    fn should_store_arbitrary_values() {
        let memdb = MockTreeStore::new(true);
        let mut trie = LeftRightTrie::new(memdb);

        trie.insert("abcdefg", CustomValue { data: 100 });

        let value: CustomValue = trie.handle().get(&String::from("abcdefg")).unwrap();

        assert_eq!(value, CustomValue { data: 100 });
    }

    #[test]
    fn should_be_read_concurrently() {
        let memdb = Arc::new(MockTreeStore::new(true));
        let mut trie = LeftRightTrie::new(memdb);

        let total = 18;

        for n in 0..total {
            let key = format!("test-{n}");

            trie.insert(key, CustomValue { data: 12345 });
        }

        trie.publish();

        // NOTE Spawn 10 threads and 10 readers that should report the exact same value
        [0..10]
            .iter()
            .map(|_| {
                let reader = trie.handle();
                thread::spawn(move || {
                    assert_eq!(reader.len(), total);
                    for n in 0..total {
                        let key = format!("test-{n}");

                        let res: CustomValue = reader.get(&key).unwrap();

                        assert_eq!(res, CustomValue { data: 12345 });
                    }
                })
            })
            .for_each(|handle| {
                handle.join().unwrap();
            });
    }
}

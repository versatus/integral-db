//! WIP Generic storage adapter
//
// TODO:
// - Finish trait implementations for PebbleDB
// - Implement namespaces for PebbleDB, similarly to MicroKV & RocksDB
// - Create test module for PebbleDB
// - Abstract PebbleDB into seperate crate
use anyhow::Result;
use parking_lot::RwLock;
use patriecia::{
    KeyHash, Node, NodeKey, OwnedValue, Preimage, StaleNodeIndex, TreeReader, TreeUpdateBatch,
    TreeWriter, Vers, VersionedDatabase,
};
use std::{
    collections::{hash_map::IntoIter, BTreeMap, BTreeSet, HashMap},
    path::PathBuf,
    sync::Arc,
};

/// A mix of MicroKV's approach to storage with the byte vec storage types of RocksDB
//
// Note: This may actually prove useful for more than testing since we will want
//       similar functionality to RocksDB without the overhead cost of a full instance.
//
// TODO: determine method for column family implementation
#[derive(Clone, Default, Debug)]
pub struct PebbleDB {
    inner: indexmap::IndexMap<Vec<u8>, Vec<u8>>,
    // cfs: BTreeMap<String, ColumnFamily>,
    path: PathBuf,
}
impl PebbleDB {
    pub fn inner(&self) -> &indexmap::IndexMap<Vec<u8>, Vec<u8>> {
        &self.inner
    }
    // pub fn cfs(&self) -> &BTreeMap<String, ColumnFamily> {
    //     &self.cfs
    // }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Intermediate trait for getting an iterator over the entire storage generically.
///
/// Coerce an individual DB's iterator into an iterator over (Vec<u8>, Vec<u8>) since we
/// only want the Ok(value)s and convert them to Vec anyway.
///
/// TLDR; This simplifies iteration of any DB so we can use any DB we want.
pub trait DiskIter: Send + Sync + std::fmt::Debug + Default + Clone {
    type DiskIterator: Iterator<Item = (Vec<u8>, Vec<u8>)>;
    fn iter(&self) -> Self::DiskIterator;
}
impl DiskIter for PebbleDB {
    type DiskIterator = indexmap::map::IntoIter<Vec<u8>, Vec<u8>>;
    fn iter(&self) -> Self::DiskIterator {
        self.inner.clone().into_iter()
    }
}

/// A generic database adapter.
#[derive(Debug, Default, Clone)]
pub struct DbAdapter<D: DiskIter> {
    data: Arc<RwLock<DbInner<D>>>,
    column: String,
}

/// The underlying generic storage.
#[derive(Debug, Default, Clone)]
pub struct DbInner<D: DiskIter> {
    db: D,
    // TODO: Determine if these fields can
    // be managed within the backing db
    stale_nodes: BTreeSet<StaleNodeIndex>,
    value_history: HashMap<KeyHash, Vec<(Vers, Option<OwnedValue>)>>,
    preimages: HashMap<KeyHash, Preimage>,
}

impl<D: DiskIter> VersionedDatabase for DbAdapter<D> {
    type Version = Vers;
    type NodeIter = IntoIter<NodeKey, Node>;
    type HistoryIter = IntoIter<patriecia::KeyHash, Vec<(Vers, Option<OwnedValue>)>>;

    fn get(&self, max_version: Self::Version, node_key: KeyHash) -> Result<Option<OwnedValue>> {
        todo!()
    }

    fn update_batch(&self, tree_update_batch: TreeUpdateBatch) -> Result<()> {
        todo!()
    }

    fn nodes(&self) -> IntoIter<NodeKey, Node> {
        let locked = self.data.read();
        let iter = locked.db.iter();
        let mut map = HashMap::new();
        for (key_bytes, node_bytes) in iter {
            if let Ok(node_key) = bincode::deserialize::<NodeKey>(&key_bytes) {
                if let Ok(node) = bincode::deserialize::<Node>(&node_bytes) {
                    map.insert(node_key, node);
                }
            };
        }

        map.into_iter()
    }

    fn value_history(
        &self,
    ) -> std::collections::hash_map::IntoIter<
        patriecia::KeyHash,
        Vec<(Self::Version, Option<patriecia::OwnedValue>)>,
    > {
        self.data.read().value_history.clone().into_iter()
    }
}

impl<D: DiskIter> TreeReader for DbAdapter<D> {
    type Version = Vers;

    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        todo!()
    }

    fn get_value_option(
        &self,
        max_version: Self::Version,
        key_hash: KeyHash,
    ) -> Result<Option<OwnedValue>> {
        todo!()
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, patriecia::LeafNode)>> {
        todo!()
    }
}
impl<D: DiskIter> TreeWriter for DbAdapter<D> {
    fn write_node_batch(&self, node_batch: &patriecia::NodeBatch) -> Result<()> {
        todo!()
    }
}

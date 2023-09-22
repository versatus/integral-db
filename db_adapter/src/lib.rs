//! WIP Generic storage adapter
//
// TODO:
// - Finish trait implementations for PebbleDB
// - Implement namespaces for PebbleDB, similarly to MicroKV & RocksDB
// - Create test module for PebbleDB
// - Abstract PebbleDB into seperate crate
use anyhow::{bail, Result};
use fnv::FnvHasher as DefaultHasher;
use parking_lot::RwLock;
use patriecia::{
    KeyHash, Node, NodeKey, OwnedValue, Preimage, StaleNodeIndex, TreeReader, TreeUpdateBatch,
    TreeWriter, Vers, VersionedDatabase,
};
use serde::Serialize;
use std::{
    collections::{hash_map::IntoIter, BTreeMap, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::Arc,
    vec,
};

const DEFAULT_COLUMN_FAMILY: &str = "default";

/// Holds the name of a column family.
///
/// Easily craft a new column family from a string or bytes.
///
/// Example:
/// ```rust
/// use db_adapter::{ColumnFamily, PebbleDB};
/// let mut db = PebbleDB::default();
///
/// db.insert(("key", "val"), &"cf".into());
/// assert_eq!(db.cf_len(&"cf".into()), Some(1));
/// ```
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ColumnFamily(String);
impl ColumnFamily {
    /// Create a key prefix hash. Output is always 8 bytes long.
    pub fn cf_key(&self) -> Result<ColumnKey> {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        Ok(hasher.finish().to_be_bytes().to_vec())
    }
}
#[cfg(test)]
mod column_family_tests {
    use crate::ColumnFamily;
    use fnv::FnvHasher as DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn __test_hash_helper() -> Vec<u8> {
        let mut hasher = DefaultHasher::default();
        ColumnFamily::default().hash(&mut hasher);
        dbg!(hasher.finish().to_be_bytes().to_vec())
    }

    #[test]
    fn test_cf_key_output_is_verifiable() {
        let cf = ColumnFamily::default();
        assert_eq!(cf.cf_key().unwrap(), __test_hash_helper());
    }
}

impl From<String> for ColumnFamily {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl<'a> From<&'a str> for ColumnFamily {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}
impl<'a> From<&'a [u8]> for ColumnFamily {
    /// Use with caution. Returns the default if it fails to parse.
    fn from(value: &'a [u8]) -> Self {
        Self::from(value.to_vec())
    }
}
impl From<Vec<u8>> for ColumnFamily {
    /// Use with caution. Returns the default if it fails to parse.
    fn from(value: Vec<u8>) -> Self {
        match String::from_utf8(value) {
            Ok(s) => Self(s),
            Err(e) => {
                println!("failed to convert bytes from utf8 into column name: {e}");
                println!("the default column family name will be used");
                Self::default()
            }
        }
    }
}
impl Default for ColumnFamily {
    fn default() -> Self {
        Self::from(DEFAULT_COLUMN_FAMILY)
    }
}
/// A hashed ColumnFamily, always 8 bytes in length.
type ColumnKey = Vec<u8>;
type PrefixedKey = Vec<u8>;

/// A mix of MicroKV's approach to storage with the byte vec storage types of RocksDB.
///
/// Note: the default column family will always exist to maintain storage logic.
//
// Note: This may actually prove useful for more than testing since we will want
//       similar functionality to RocksDB without the overhead cost of a full instance.
#[derive(Clone, Debug)]
pub struct PebbleDB {
    inner: indexmap::IndexMap<PrefixedKey, OwnedValue>,
    /// A map of the column family names and the keys with
    /// the first 8 bytes being the hashed column family name
    ///
    /// This is separate from the inner store in order to
    /// keep all pairs in a single structure
    ///
    /// A nice side effect of this structure is that
    /// if a duplicate column family is provided it
    /// won't return an error and instead just update
    /// the corresponding vec of prefixed keys
    cfs: BTreeMap<ColumnFamily, Vec<PrefixedKey>>,
    path: PathBuf,
}
impl Default for PebbleDB {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            cfs: Default::default(),
            path: PebbleDB::get_db_path(),
        }
    }
}
impl PebbleDB {
    pub fn inner(&self) -> &indexmap::IndexMap<Vec<u8>, Vec<u8>> {
        &self.inner
    }
    pub fn cfs(&self) -> &BTreeMap<ColumnFamily, Vec<PrefixedKey>> {
        &self.cfs
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    /// Helper that retrieves the home directory by resolving $HOME
    #[inline]
    fn get_home_dir() -> PathBuf {
        dirs::home_dir().unwrap()
    }

    /// Helper that forms an absolute path from a given database name and the default workspace path.
    #[inline]
    pub fn get_db_path() -> PathBuf {
        const DEFAULT_WORKSPACE_PATH: &str = ".pebbledb/";
        let mut path = PebbleDB::get_home_dir();
        path.push(DEFAULT_WORKSPACE_PATH);
        path.push("versatus");
        path.set_extension("pb");
        path
    }

    pub fn cf_exists(&self, cf: &ColumnFamily) -> bool {
        self.cfs().contains_key(cf)
    }
    /// Initialized a new ColumnFamily with a None value.
    pub fn new_cf(&mut self, cf: &ColumnFamily) -> Result<()> {
        if !self.cf_exists(cf) {
            self.cfs.insert(cf.clone(), vec![]);
            return Ok(());
        }
        println!("failed to create new column family from: {cf:?}\ncolumn family already exists");
        Ok(())
    }
    pub fn insert<K, V>(&mut self, kv: (K, V), cf: &ColumnFamily) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        let mut key = bincode::serialize(&kv.0)?;
        let mut cf_key = cf.cf_key()?;
        cf_key.append(&mut key);

        self.inner
            .insert(cf_key.clone(), bincode::serialize(&kv.1)?);

        if let Some(cf_keys) = self.cfs.get_mut(cf) {
            if !cf_keys.contains(&cf_key) {
                cf_keys.push(cf_key);
            }
        } else {
            self.cfs.insert(cf.clone(), vec![cf_key]);
        }

        Ok(())
    }
    /// Return the length of a column family's prefixed keys if the column family exists
    pub fn cf_len(&self, cf: &ColumnFamily) -> Option<usize> {
        self.cfs().get(cf).and_then(|cf| Some(cf.len()))
    }
}

#[cfg(test)]
mod pebble_db_tests {
    use crate::{ColumnFamily, PebbleDB};

    #[test]
    fn test_insert() {
        let mut db = PebbleDB::default();
        dbg!(&db);

        db.insert(("node_id1", "claim1"), &ColumnFamily::from("claims"))
            .unwrap();
        dbg!(&db);
        assert_eq!(db.cfs().len(), 1);
        assert!(db.cf_exists(&"claims".into()));

        db.insert(("node_id2", "claim2"), &ColumnFamily::from("claims"))
            .unwrap();
        dbg!(&db);
        assert_eq!(db.cf_len(&"claims".into()).unwrap(), 2);

        db.insert(("address1", "account1"), &ColumnFamily::from("state"))
            .unwrap();
        dbg!(&db);
        assert_eq!(db.cfs().len(), 2);
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

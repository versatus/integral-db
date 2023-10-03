use crate::{
    column_family::{ColumnFamily, PrefixedKey},
    DiskIter,
};
use anyhow::Result;
use patriecia::OwnedValue;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

/// A mix of MicroKV's approach to storage with the byte vec storage types of RocksDB.
///
/// Note: the default column family will always exist to maintain storage logic.
//
// Note: This may actually prove useful for more than testing since we will want
//       similar functionality to RocksDB without the overhead cost of a full instance.
//
// TODO: Make keys and values generic on the inner map?
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// Inserts key value pairs into the storage map. This also updates the column family map.
    ///
    /// The keys of the storage map are appended to the hash of the column family, allowing for
    /// a single point of storage for multiple namespaces. This is also a point of sanity, to allow
    /// for verification that prefixed keyhashes are present, or not present.
    ///
    /// If a key is present in the storage map, it's value is updated in place otherwise appended to the map.
    ///
    /// If a column family exists, the prefixed key is appended to the column family map's associated prefixed keys.
    /// Otherwise it is inserted as a new entry.
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
    /// Return the length of a column family's prefixed keys if the column family exists.
    ///
    /// Note: If added all together, the number of values in the column family keys
    ///       should equal the number of keys in the the storage map.
    pub fn cf_len(&self, cf: &ColumnFamily) -> Option<usize> {
        self.cfs().get(cf).and_then(|cf| Some(cf.len()))
    }
}
impl DiskIter for PebbleDB {
    type DiskIterator = indexmap::map::IntoIter<Vec<u8>, Vec<u8>>;
    fn iter(&self) -> Self::DiskIterator {
        self.inner.clone().into_iter()
    }
}

#[cfg(test)]
mod pebble_db_tests {
    use crate::{column_family::ColumnFamily, pebble_db::PebbleDB};

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
        assert_eq!(
            db.cf_len(&"claims".into()).unwrap() + db.cf_len(&"state".into()).unwrap(),
            db.inner.values().into_iter().len()
        );
    }
}

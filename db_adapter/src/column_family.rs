use anyhow::Result;
use fnv::FnvHasher as DefaultHasher;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

const DEFAULT_COLUMN_FAMILY: &str = "default";
/// A hashed ColumnFamily, always 8 bytes in length.
pub(crate) type ColumnKey = Vec<u8>;
pub(crate) type PrefixedKey = Vec<u8>;

/// Holds the name of a column family.
///
/// Easily craft a new column family from a string or bytes.
///
/// Example:
/// ```rust
/// use db_adapter::{column_family::{ColumnFamily}, pebble_db::PebbleDB};
/// let mut db = PebbleDB::default();
///
/// db.insert(("key", "val"), &"cf".into());
/// assert_eq!(db.cf_len(&"cf".into()), Some(1));
/// ```
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ColumnFamily(String);
impl ColumnFamily {
    /// Create a key prefix hash. Output is always 8 bytes long.
    pub fn cf_key(&self) -> Result<ColumnKey> {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        Ok(hasher.finish().to_be_bytes().to_vec())
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

#[cfg(test)]
mod column_family_tests {
    use crate::column_family::ColumnFamily;
    use fnv::FnvHasher as DefaultHasher;
    use std::hash::{Hash, Hasher};

    // verify a second instance of the default hasher
    // and default column family produce the same hash
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

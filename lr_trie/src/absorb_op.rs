use crate::Operation;
use left_right::Absorb;
pub use left_right::ReadHandleFactory;
use patriecia::{
    JellyfishMerkleTree, SimpleHasher, TreeReader, TreeWriter, VersionedDatabase, VersionedTrie,
};
use tracing::error;

/// The number by which the [`Version`] of a [`JellyfishMerkleTree`] is incremented.
const INCREMENT_ARG: u64 = 1;

impl<D, H> Absorb<Operation> for JellyfishMerkleTree<D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    fn absorb_first(&mut self, operation: &mut Operation, _other: &Self) {
        // TODO: the unwrap_or_default avoids panic but is not logically sound and should
        // not be used in production since `Version` is monotonically increasing and could
        // eventually overflow.
        //
        // cc @nopestack
        let increment_version =
            |vers: &mut u64| vers.checked_add(INCREMENT_ARG).unwrap_or_default();
        match operation {
            // TODO: report errors via instrumentation
            Operation::Add(key_val, vers) => {
                match self.put_value_set(vec![key_val.to_owned()], increment_version(vers)) {
                    Ok((_, batch)) => {
                        if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                            error!("Operation::Add failed to write changes to database: {err}")
                        }
                    }
                    Err(err) => error!("Operation::Add failed to insert key: {err}"),
                }
            }
            Operation::Remove(key, vers) => {
                match self.put_value_set(vec![(*key, None)], increment_version(vers)) {
                    Ok((_, batch)) => {
                        if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                            error!("Operation::Remove failed to write changes to database: {err}")
                        }
                    }
                    Err(err) => error!("Operation::Remove failed to remove value for key: {err}"),
                }
            }
            Operation::Extend(kvs, vers) => {
                match self.put_value_set(kvs.to_vec(), increment_version(vers)) {
                    Ok((_, batch)) => {
                        if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                            error!("Operation::Extend failed to write changes to database: {err}")
                        }
                    }
                    Err(err) => error!("Operation::Extend failed batch update: {err}"),
                }
            }
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

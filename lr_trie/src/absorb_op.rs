use left_right::Absorb;
pub use left_right::ReadHandleFactory;
use patriecia::{
    JellyfishMerkleTree, SimpleHasher, TreeReader, TreeWriter, VersionedDatabase, VersionedTrie,
};
use tracing::error;

use crate::Operation;

impl<D, H> Absorb<Operation> for JellyfishMerkleTree<D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    fn absorb_first(&mut self, operation: &mut Operation, _other: &Self) {
        match operation {
            // TODO: report errors via instrumentation
            Operation::Add(key_val, vers) => {
                match self.put_value_set(vec![key_val.to_owned()], *vers + 1) {
                    Ok((_, batch)) => {
                        if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                            error!("Operation::Add failed to write changes to database: {err}")
                        }
                    }
                    Err(err) => error!("Operation::Add failed to insert key: {err}"),
                }
            }
            Operation::Remove(key, vers) => match self.put_value_set(vec![(*key, None)], *vers + 1)
            {
                Ok((_, batch)) => {
                    if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                        error!("Operation::Remove failed to write changes to database: {err}")
                    }
                }
                Err(err) => error!("Operation::Remove failed to remove value for key: {err}"),
            },
            Operation::Extend(kvs, vers) => match self.put_value_set(kvs.to_vec(), *vers + 1) {
                Ok((_, batch)) => {
                    if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                        error!("Operation::Extend failed to write changes to database: {err}")
                    }
                }
                Err(err) => error!("Operation::Extend failed batch update: {err}"),
            },
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

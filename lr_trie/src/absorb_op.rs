use left_right::Absorb;
pub use left_right::ReadHandleFactory;
use patriecia::{
    JellyfishMerkleTree, SimpleHasher, TreeReader, TreeWriter, VersionedDatabase, VersionedTrie,
};
use tracing::error;

use crate::Operation;

impl<'a, D, H> Absorb<Operation> for JellyfishMerkleTree<'a, D, H>
where
    D: TreeReader + TreeWriter + VersionedDatabase,
    H: SimpleHasher,
{
    fn absorb_first(&mut self, operation: &mut Operation, _other: &Self) {
        match operation {
            // TODO: report errors via instrumentation
            Operation::Add(key_val, vers) => {
                match self.put_value_set(vec![key_val.to_owned()], *vers) {
                    Ok((_, batch)) => {
                        if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                            error!("Operation::Add failed to write changes to database: {err}")
                        }
                    }
                    Err(err) => error!("Operation::Add failed to insert key: {err}"),
                }
            }
            Operation::Remove(key, vers) => match self.put_value_set(vec![(*key, None)], *vers) {
                Ok((_, batch)) => {
                    if let Err(err) = self.reader().write_node_batch(&batch.node_batch) {
                        error!("Operation::Remove failed to write changes to database: {err}")
                    }
                }
                Err(err) => error!("Operation::Remove failed to remove value for key: {err}"),
            },
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

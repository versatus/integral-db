use left_right::Absorb;
pub use left_right::ReadHandleFactory;
use patriecia::{JellyfishMerkleTree, SimpleHasher, TreeReader, VersionedDatabase};
use tracing::error;

use crate::Operation;

impl<'a, D, H> Absorb<Operation> for JellyfishMerkleTree<'a, D, H>
where
    D: TreeReader + VersionedDatabase,
    H: SimpleHasher,
{
    fn absorb_first(&mut self, operation: &mut Operation, _other: &Self) {
        match operation {
            // TODO: report errors via instrumentation
            Operation::Add(key_val, vers) => {
                if let Err(err) = self.put_value_set(vec![key_val.to_owned()], *vers) {
                    error!("failed to insert key: {err}");
                }
            }
            Operation::Remove(key, vers) => {
                if let Err(err) = self.put_value_set(vec![(*key, None)], *vers) {
                    error!("failed to remove value for key: {err}");
                }
            }
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

use left_right::Absorb;
pub use left_right::ReadHandleFactory;
use patriecia::{
    db::Database, trie::Trie, JellyfishMerkleTree, SimpleHasher, TreeReader, VersionedDatabase,
};
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
            Operation::Add(key, value) => {
                if let Err(err) = self.insert(key, value) {
                    error!("failed to insert key: {err}");
                }
            }
            Operation::Remove(key) => {
                if let Err(err) = self.remove(key) {
                    error!("failed to remove value for key: {err}");
                }
            }
            Operation::Extend(values) => {
                //
                // TODO: temp hack to get this going. Refactor ASAP
                //
                for (k, v) in values {
                    if let Err(err) = self.insert(k, v) {
                        error!("failed to insert key: {err}");
                    }
                }
            }
        }

        if let Err(err) = self.commit() {
            error!("failed to commit changes to trie: {err}");
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
        if let Err(err) = self.commit() {
            tracing::error!("failed to commit changes to trie: {err}");
        }
    }
}

use std::sync::Arc;
use super::{
    ClientDB,
    DBError,
};
use super::BLOCKS_DB_COLUMN as DB_COLUMN;

pub struct BlockStore<T>
    where T: ClientDB
{
    db: Arc<T>,
}

impl<T: ClientDB> BlockStore<T> {
    pub fn new(db: Arc<T>) -> Self {
        Self {
            db,
        }
    }

    pub fn put_serialized_block(&self, hash: &[u8], ssz: &[u8])
        -> Result<(), DBError>
    {
        self.db.put(DB_COLUMN, hash, ssz)
    }

    pub fn get_serialized_block(&self, hash: &[u8])
        -> Result<Option<Vec<u8>>, DBError>
    {
        self.db.get(DB_COLUMN, hash)
    }

    pub fn block_exists(&self, hash: &[u8])
        -> Result<bool, DBError>
    {
        self.db.exists(DB_COLUMN, hash)
    }

    pub fn block_exists_in_canonical_chain(&self, hash: &[u8])
        -> Result<bool, DBError>
    {
        // TODO: implement logic for canonical chain
        self.db.exists(DB_COLUMN, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::MemoryDB;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_block_store_on_disk_db() {
        let db = Arc::new(MemoryDB::open());
        let bs = Arc::new(BlockStore::new(db.clone()));

        let thread_count = 10;
        let write_count = 10;

        // We're expecting the product of these numbers to fit in one byte.
        assert!(thread_count * write_count <= 255);

        let mut handles = vec![];
        for t in 0..thread_count {
            let wc = write_count;
            let bs = bs.clone();
            let handle = thread::spawn(move || {
                for w in 0..wc {
                    let key = (t * w) as u8;
                    let val = 42;
                    bs.put_serialized_block(&vec![key], &vec![val]).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for t in 0..thread_count {
            for w in 0..write_count {
                let key = (t * w) as u8;
                assert!(bs.block_exists(&vec![key]).unwrap());
                let val = bs.get_serialized_block(&vec![key]).unwrap().unwrap();
                assert_eq!(vec![42], val);
            }
        }
    }
}

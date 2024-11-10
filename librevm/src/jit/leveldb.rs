use db_key::Key;
use leveldb::database::Database;
use leveldb::iterator::{Iterable, KeyIterator};
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use revmc::eyre::{self, Result};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

// phantom data to use lifetime parameter
pub struct LevelDB<'a, K>
where
    K: 'a + Key,
{
    pub db: Arc<Database<K>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, K> LevelDB<'a, K>
where
    K: 'a + Key,
{
    pub fn init(path: &'a str) -> Self {
        let db = LevelDB::connect(path).unwrap();
        Self {
            db: Arc::new(db),
            _marker: PhantomData,
        }
    }

    fn connect(path: &str) -> Result<Database<K>> {
        let mut options = Options::new();
        options.create_if_missing = true;

        Database::open(Path::new(path), options).map_err(|e| eyre::Report::new(e))
    }

    pub fn put(&self, key: K, value: &[u8], sync: bool) -> Result<()> {
        let mut write_options = WriteOptions::new();
        if sync {
            write_options.sync = true;
        }

        self.db
            .put(write_options, key, value)
            .map_err(|e| eyre::Report::new(e))
    }

    pub fn get(&self, key: K) -> Result<Option<Vec<u8>>> {
        let read_options = ReadOptions::new();
        self.db
            .get(read_options, key)
            .map_err(|e| eyre::Report::new(e))
    }

    pub fn key_iterator(&self) -> KeyIterator<K> {
        let read_options = ReadOptions::new();
        let keys_iter = self.db.keys_iter(read_options);

        keys_iter
    }
}

impl<'a, K> Clone for LevelDB<'a, K>
where
    K: 'a + Key,
{
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            _marker: PhantomData,
        }
    }
}

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;

use rocksdb::{IteratorMode, Options as RocksDBOptions, WriteBatch, DB as RocksDB};

use clipcat::ClipboardData;

use crate::history::{HistoryDriver, HistoryError};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardValue {
    pub data: String,
    pub timestamp: SystemTime,
}

pub struct RocksDBDriver {
    db: Option<RocksDB>,
}

impl RocksDBDriver {
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<RocksDBDriver, HistoryError> {
        let opt = Self::open_options();
        let db = RocksDB::open(&opt, file_path)?;
        Ok(RocksDBDriver { db: Some(db) })
    }

    fn open_options() -> RocksDBOptions {
        let mut opt = RocksDBOptions::default();
        opt.create_if_missing(true);
        opt
    }

    fn serialize_id(id: u64) -> Vec<u8> {
        bincode::serialize(&id).expect("u64 is serializable")
    }

    fn deserialize_id(id: &[u8]) -> u64 {
        bincode::deserialize(&id).expect("u64 is deserializable")
    }

    fn deserialize_data(id: u64, raw_data: &[u8]) -> Option<ClipboardData> {
        use clipcat::ClipboardType;

        bincode::deserialize::<ClipboardValue>(&raw_data)
            .map(|value| ClipboardData {
                id,
                data: value.data.clone(),
                timestamp: value.timestamp,
                clipboard_type: ClipboardType::Primary,
            })
            .map_err(|_| {
                warn!("Failed to deserialize ClipboardValue");
            })
            .ok()
    }

    fn serialize_data(data: &ClipboardData) -> Vec<u8> {
        let value = ClipboardValue { data: data.data.clone(), timestamp: data.timestamp };
        bincode::serialize(&value).expect("ClipboardData is serializable")
    }

    fn serialize_entry(id: u64, data: &ClipboardData) -> (Vec<u8>, Vec<u8>) {
        (Self::serialize_id(id), Self::serialize_data(data))
    }

    fn deserialize_entry(id: &[u8], data: &[u8]) -> Option<ClipboardData> {
        let id = Self::deserialize_id(id);
        Self::deserialize_data(id, data)
    }
}

impl HistoryDriver for RocksDBDriver {
    fn load(&self) -> Result<Vec<ClipboardData>, HistoryError> {
        let db = self.db.as_ref().expect("RocksDB must be some");
        let iter = db.iterator(IteratorMode::Start);
        let clips = iter
            .filter_map(|(id, data)| Self::deserialize_entry(id.as_ref(), data.as_ref()))
            .collect();
        Ok(clips)
    }

    fn save(&mut self, data: &Vec<ClipboardData>) -> Result<(), HistoryError> {
        let db = self.db.as_mut().expect("RocksDB must be some");

        let iter = db.iterator(IteratorMode::Start);
        let ids_in_db: HashSet<Vec<u8>> = iter.map(|(k, _v)| k.into_vec()).collect();

        let mut batch = WriteBatch::default();
        let unsaved_ids: HashSet<_> = data
            .iter()
            .map(|clip| {
                let (id, data) = Self::serialize_entry(clip.id, &clip);
                batch.put(id.clone(), data);
                id
            })
            .collect();

        ids_in_db.difference(&unsaved_ids).for_each(|id| {
            let _ = db.delete(id);
        });

        db.write(batch)?;
        Ok(())
    }

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), HistoryError> {
        let db = self.db.as_mut().expect("RocksDB must be some");
        if db.iterator(IteratorMode::Start).count() < min_capacity {
            return Ok(());
        }

        let iter = db.iterator(IteratorMode::Start);
        let timestamps = iter
            .filter_map(|(k, v)| {
                let id = Self::deserialize_id(&k);
                let v = Self::deserialize_data(id, &v);
                v.map(|v| (v, Vec::from(k.as_ref())))
            })
            .map(|(v, id)| (v.timestamp, id))
            .collect::<HashMap<SystemTime, Vec<u8>>>();

        let batch = {
            let mut keys = timestamps.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let len = keys.len();
            keys.resize(len - min_capacity, SystemTime::now());
            keys.iter().filter_map(|ts| timestamps.get(&ts)).fold(
                WriteBatch::default(),
                |mut batch, id| {
                    batch.delete(id);
                    batch
                },
            )
        };

        db.write(batch)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), HistoryError> {
        let db_path = {
            let db = self.db.take().expect("RocksDB must be some");
            let db_path = db.path().to_path_buf();
            drop(db);
            db_path
        };

        RocksDB::destroy(&RocksDBOptions::default(), &db_path)?;
        self.db = Some(RocksDB::open(&Self::open_options(), &db_path)?);
        Ok(())
    }

    fn put(&mut self, data: &ClipboardData) -> Result<(), HistoryError> {
        let db = self.db.as_mut().expect("RocksDB must be some");
        db.put(Self::serialize_id(data.id), Self::serialize_data(&data))?;
        Ok(())
    }

    fn get(&self, id: u64) -> Result<Option<ClipboardData>, HistoryError> {
        let db = self.db.as_ref().expect("RocksDB must be some");
        let serialized_id = Self::serialize_id(id);
        match db.get(&serialized_id)? {
            Some(data) => Ok(Self::deserialize_data(id, &data)),
            None => Ok(None),
        }
    }
}

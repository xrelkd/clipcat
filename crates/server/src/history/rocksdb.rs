use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::SystemTime,
};

use clipcat::ClipEntry;
use rocksdb::{IteratorMode, Options as RocksDBOptions, WriteBatch, DB as RocksDB};

use crate::history::{Error, HistoryDriver};

mod v2 {
    use std::time::SystemTime;

    use clipcat::{utils, ClipEntry, ClipboardMode};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ClipboardValue {
        pub data: Vec<u8>,
        #[serde(
            serialize_with = "utils::serialize_mime",
            deserialize_with = "utils::deserialize_mime"
        )]
        pub mime: mime::Mime,
        pub timestamp: SystemTime,
    }

    impl ClipboardValue {
        pub fn into_data(self, id: u64) -> ClipEntry {
            ClipEntry {
                id,
                data: self.data,
                mime: self.mime,
                timestamp: self.timestamp,
                mode: ClipboardMode::Selection,
            }
        }
    }

    impl From<ClipEntry> for ClipboardValue {
        fn from(data: ClipEntry) -> Self {
            let ClipEntry { data, mime, timestamp, .. } = data;
            Self { data, mime, timestamp }
        }
    }
}

mod v1 {
    use std::time::SystemTime;

    use clipcat::{ClipEntry, ClipboardMode};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ClipboardValue {
        pub data: String,
        pub timestamp: SystemTime,
    }

    impl ClipboardValue {
        pub fn into_data(self, id: u64) -> ClipEntry {
            ClipEntry {
                id,
                data: Vec::from(self.data.as_bytes()),
                mime: mime::TEXT_PLAIN_UTF_8,
                timestamp: self.timestamp,
                mode: ClipboardMode::Selection,
            }
        }
    }
}

pub struct RocksDBDriver {
    db: Option<RocksDB>,
}

impl RocksDBDriver {
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
        let opt = Self::open_options();
        let db = RocksDB::open(&opt, file_path)?;
        Ok(Self { db: Some(db) })
    }

    fn open_options() -> RocksDBOptions {
        let mut opt = RocksDBOptions::default();
        opt.create_if_missing(true);
        opt
    }

    fn serialize_id(id: u64) -> Vec<u8> { bincode::serialize(&id).expect("u64 is serializable") }

    fn deserialize_id(id: &[u8]) -> u64 { bincode::deserialize(id).expect("u64 is deserializable") }

    fn deserialize_data(id: u64, raw_data: &[u8]) -> Option<ClipEntry> {
        if let Ok(data) =
            bincode::deserialize::<v2::ClipboardValue>(raw_data).map(|value| value.into_data(id))
        {
            return Some(data);
        }

        tracing::info!("Try to deserialize with v1::ClipboardValue");
        bincode::deserialize::<v1::ClipboardValue>(raw_data)
            .map(|value| value.into_data(id))
            .map_err(|_| {
                tracing::warn!("Failed to deserialize v1::ClipboardValue");
            })
            .ok()
    }

    fn serialize_data(data: &ClipEntry) -> Vec<u8> {
        let value = v2::ClipboardValue::from(data.clone());
        bincode::serialize(&value).expect("ClipboardData is serializable")
    }

    fn serialize_entry(id: u64, data: &ClipEntry) -> (Vec<u8>, Vec<u8>) {
        (Self::serialize_id(id), Self::serialize_data(data))
    }

    fn deserialize_entry(id: &[u8], data: &[u8]) -> Option<ClipEntry> {
        let id = Self::deserialize_id(id);
        Self::deserialize_data(id, data)
    }
}

impl HistoryDriver for RocksDBDriver {
    fn load(&self) -> Result<Vec<ClipEntry>, Error> {
        let db = self.db.as_ref().expect("RocksDB must be some");
        let iter = db.iterator(IteratorMode::Start);
        let clips = iter
            .filter_map(|maybe_data| {
                if let Ok((id, data)) = maybe_data {
                    Self::deserialize_entry(id.as_ref(), data.as_ref())
                } else {
                    None
                }
            })
            .collect();
        Ok(clips)
    }

    fn save(&mut self, data: &[ClipEntry]) -> Result<(), Error> {
        let db = self.db.as_mut().expect("RocksDB must be some");

        let iter = db.iterator(IteratorMode::Start);
        let ids_in_db: HashSet<Vec<u8>> = iter
            .filter_map(
                |maybe_data| if let Ok((k, _v)) = maybe_data { Some(k.into_vec()) } else { None },
            )
            .collect();

        let mut batch = WriteBatch::default();
        let unsaved_ids: HashSet<_> = data
            .iter()
            .map(|clip| {
                let (id, data) = Self::serialize_entry(clip.id, clip);
                batch.put(id.clone(), data);
                id
            })
            .collect();

        ids_in_db.difference(&unsaved_ids).for_each(|id| {
            let _unused = db.delete(id);
        });

        db.write(batch)?;
        Ok(())
    }

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        let db = self.db.as_mut().expect("RocksDB must be some");
        if db.iterator(IteratorMode::Start).count() < min_capacity {
            return Ok(());
        }

        let iter = db.iterator(IteratorMode::Start);
        let timestamps = iter
            .filter_map(|maybe_data| {
                if let Ok((k, v)) = maybe_data {
                    let id = Self::deserialize_id(&k);
                    let v = Self::deserialize_data(id, &v);
                    v.map(|v| (v, Vec::from(k.as_ref())))
                } else {
                    None
                }
            })
            .map(|(v, id)| (v.timestamp, id))
            .collect::<HashMap<SystemTime, Vec<u8>>>();

        let batch = {
            let mut keys = timestamps.keys().copied().collect::<Vec<_>>();
            keys.sort();
            let len = keys.len();
            keys.resize(len - min_capacity, SystemTime::now());
            keys.iter().filter_map(|ts| timestamps.get(ts)).fold(
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

    fn clear(&mut self) -> Result<(), Error> {
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

    fn put(&mut self, data: &ClipEntry) -> Result<(), Error> {
        let db = self.db.as_mut().expect("RocksDB must be some");
        db.put(Self::serialize_id(data.id), Self::serialize_data(data))?;
        Ok(())
    }

    fn get(&self, id: u64) -> Result<Option<ClipEntry>, Error> {
        let db = self.db.as_ref().expect("RocksDB must be some");
        let serialized_id = Self::serialize_id(id);
        db.get(serialized_id)?.map_or(Ok(None), |data| Ok(Self::deserialize_data(id, &data)))
    }
}

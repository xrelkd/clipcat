#[derive(Debug, Snafu)]
pub enum HistoryError {
    #[snafu(display("RocksDB error: {}", source))]
    RocksDB { source: rocksdb::Error },
}

impl From<rocksdb::Error> for HistoryError {
    fn from(err: rocksdb::Error) -> HistoryError {
        HistoryError::RocksDB { source: err }
    }
}

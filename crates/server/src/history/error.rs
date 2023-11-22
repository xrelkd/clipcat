use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("RocksDB error: {source}",))]
    RocksDB { source: rocksdb::Error },
}

impl From<rocksdb::Error> for Error {
    fn from(source: rocksdb::Error) -> Self { Self::RocksDB { source } }
}

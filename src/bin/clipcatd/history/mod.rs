use std::path::{Path, PathBuf};

use clipcat::ClipEntry;

mod error;
mod rocksdb;

pub use self::{error::HistoryError, rocksdb::RocksDBDriver};

pub trait HistoryDriver: Send + Sync {
    fn load(&self) -> Result<Vec<ClipEntry>, HistoryError>;

    fn save(&mut self, data: &[ClipEntry]) -> Result<(), HistoryError>;

    fn clear(&mut self) -> Result<(), HistoryError>;

    fn put(&mut self, data: &ClipEntry) -> Result<(), HistoryError>;

    fn get(&self, id: u64) -> Result<Option<ClipEntry>, HistoryError>;

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), HistoryError>;

    fn save_and_shrink_to(
        &mut self,
        data: &[ClipEntry],
        min_capacity: usize,
    ) -> Result<(), HistoryError> {
        self.save(data)?;
        self.shrink_to(min_capacity)
    }
}

pub struct HistoryManager {
    file_path: PathBuf,
    driver: Box<dyn HistoryDriver>,
}

impl HistoryManager {
    #[inline]
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<HistoryManager, HistoryError> {
        let driver = Box::new(RocksDBDriver::open(&file_path)?);
        let file_path = file_path.as_ref().to_owned();
        Ok(HistoryManager { driver, file_path })
    }

    #[inline]
    pub fn path(&self) -> &Path { &self.file_path }

    #[inline]
    pub fn put(&mut self, data: &ClipEntry) -> Result<(), HistoryError> { self.driver.put(data) }

    #[inline]
    #[allow(dead_code)]
    pub fn clear(&mut self) -> Result<(), HistoryError> { self.driver.clear() }

    #[inline]
    pub fn load(&self) -> Result<Vec<ClipEntry>, HistoryError> { self.driver.load() }

    #[inline]
    pub fn save(&mut self, data: &[ClipEntry]) -> Result<(), HistoryError> {
        self.driver.save(data)
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) -> Result<(), HistoryError> {
        self.driver.shrink_to(min_capacity)
    }

    #[inline]
    pub fn save_and_shrink_to(
        &mut self,
        data: &[ClipEntry],
        min_capacity: usize,
    ) -> Result<(), HistoryError> {
        self.save(data)?;
        self.shrink_to(min_capacity)
    }
}

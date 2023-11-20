mod error;
mod rocksdb;

use std::path::{Path, PathBuf};

use clipcat::ClipEntry;

pub use self::{error::Error, rocksdb::RocksDBDriver};

pub trait HistoryDriver: Send + Sync {
    fn load(&self) -> Result<Vec<ClipEntry>, Error>;

    fn save(&mut self, data: &[ClipEntry]) -> Result<(), Error>;

    fn clear(&mut self) -> Result<(), Error>;

    fn put(&mut self, data: &ClipEntry) -> Result<(), Error>;

    fn get(&self, id: u64) -> Result<Option<ClipEntry>, Error>;

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error>;

    fn save_and_shrink_to(&mut self, data: &[ClipEntry], min_capacity: usize) -> Result<(), Error> {
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
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
        let driver = Box::new(RocksDBDriver::open(&file_path)?);
        let file_path = file_path.as_ref().to_owned();
        Ok(Self { file_path, driver })
    }

    #[inline]
    pub fn path(&self) -> &Path { &self.file_path }

    #[inline]
    pub fn put(&mut self, data: &ClipEntry) -> Result<(), Error> { self.driver.put(data) }

    #[inline]
    #[allow(dead_code)]
    pub fn clear(&mut self) -> Result<(), Error> { self.driver.clear() }

    #[inline]
    pub fn load(&self) -> Result<Vec<ClipEntry>, Error> { self.driver.load() }

    #[inline]
    pub fn save(&mut self, data: &[ClipEntry]) -> Result<(), Error> { self.driver.save(data) }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        self.driver.shrink_to(min_capacity)
    }

    #[inline]
    pub fn save_and_shrink_to(
        &mut self,
        data: &[ClipEntry],
        min_capacity: usize,
    ) -> Result<(), Error> {
        self.save(data)?;
        self.shrink_to(min_capacity)
    }
}

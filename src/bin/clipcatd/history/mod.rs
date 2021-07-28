use std::path::{Path, PathBuf};

use clipcat::ClipboardData;

mod error;

mod fs;

pub use self::error::HistoryError;

pub trait HistoryDriver: Send + Sync {
    fn load(&self) -> Result<Vec<ClipboardData>, HistoryError>;

    fn save(&mut self, data: &[ClipboardData]) -> Result<(), HistoryError>;

    fn clear(&mut self) -> Result<(), HistoryError>;

    fn put(&mut self, data: &ClipboardData) -> Result<(), HistoryError>;

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), HistoryError>;

    fn save_and_shrink_to(
        &mut self,
        data: &[ClipboardData],
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
        let mut path = file_path.as_ref().to_path_buf();
        if path.is_dir() {
            path.set_file_name("history.cdb")
        }
        let driver = Box::new(fs::SimpleDBDriver::new(&path));
        Ok(HistoryManager { driver, file_path: path })
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.file_path
    }
}
impl HistoryDriver for HistoryManager {
    #[inline]
    fn put(&mut self, data: &ClipboardData) -> Result<(), HistoryError> {
        self.driver.put(data)
    }

    #[inline]
    #[allow(dead_code)]
    fn clear(&mut self) -> Result<(), HistoryError> {
        self.driver.clear()
    }

    #[inline]
    fn load(&self) -> Result<Vec<ClipboardData>, HistoryError> {
        self.driver.load()
    }

    #[inline]
    fn save(&mut self, data: &[ClipboardData]) -> Result<(), HistoryError> {
        self.driver.save(data)
    }

    #[inline]
    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), HistoryError> {
        self.driver.shrink_to(min_capacity)
    }
}

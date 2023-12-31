mod driver;
mod error;

use std::path::{Path, PathBuf};

use clipcat_base::ClipEntry;

pub use self::error::Error;

pub struct HistoryManager {
    file_path: PathBuf,
    driver: Box<dyn driver::Driver>,
}

impl HistoryManager {
    /// # Errors
    #[inline]
    pub async fn new<P>(file_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path> + Send,
    {
        let file_path = file_path.as_ref().to_owned();
        let driver = driver::FileSystemDriver::new(&file_path).await?;
        Ok(Self { file_path, driver: Box::new(driver) })
    }

    #[inline]
    pub fn path(&self) -> &Path { &self.file_path }

    #[inline]
    pub async fn put(&mut self, data: &ClipEntry) -> Result<(), Error> {
        self.driver.put(data).await
    }

    #[allow(dead_code)]
    #[inline]
    pub async fn clear(&mut self) -> Result<(), Error> { self.driver.clear().await }

    #[inline]
    pub async fn load(&mut self) -> Result<Vec<ClipEntry>, Error> { self.driver.load().await }

    #[allow(dead_code)]
    #[inline]
    pub async fn save(&mut self, data: &[ClipEntry]) -> Result<(), Error> {
        self.driver.save(data).await
    }

    #[allow(dead_code)]
    #[inline]
    pub async fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        self.driver.shrink_to(min_capacity).await
    }

    #[inline]
    pub async fn save_and_shrink_to(
        &mut self,
        data: &[ClipEntry],
        min_capacity: usize,
    ) -> Result<(), Error> {
        self.driver.save_and_shrink_to(data, min_capacity).await
    }
}

mod fs;

use async_trait::async_trait;
use clipcat_base::ClipEntry;

pub use self::fs::FileSystemDriver;
use crate::history::Error;

#[async_trait]
pub trait Driver: Send + Sync {
    async fn load(&self) -> Result<Vec<ClipEntry>, Error>;

    async fn save(&mut self, data: &[ClipEntry]) -> Result<(), Error>;

    async fn clear(&mut self) -> Result<(), Error>;

    async fn put(&mut self, data: &ClipEntry) -> Result<(), Error>;

    async fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error>;

    async fn save_and_shrink_to(
        &mut self,
        data: &[ClipEntry],
        min_capacity: usize,
    ) -> Result<(), Error> {
        self.save(data).await?;
        self.shrink_to(min_capacity).await
    }
}

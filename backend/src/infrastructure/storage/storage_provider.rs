use async_trait::async_trait;
use uuid::Uuid;

/// Abstraction over the document storage backend.
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Persist a file and return the relative path that can later be used to read/delete it.
    async fn save_file(
        &self,
        building_id: Uuid,
        filename: &str,
        content: &[u8],
    ) -> Result<String, String>;

    /// Retrieve the raw bytes and return them as a vector.
    async fn read_file(&self, relative_path: &str) -> Result<Vec<u8>, String>;

    /// Delete a file if it exists.
    async fn delete_file(&self, relative_path: &str) -> Result<(), String>;

    /// Check if a file exists in the storage backend.
    async fn file_exists(&self, relative_path: &str) -> bool;
}

use super::{metrics::record_storage_operation, StorageProvider};
use async_trait::async_trait;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

/// File storage service for managing document uploads
#[derive(Clone)]
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// Create a new FileStorage with the given base path
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self, String> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create base directory if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(&base_path)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        Ok(Self { base_path })
    }

    /// Save a file to storage and return the relative file path
    /// Files are organized by building_id: /uploads/{building_id}/{filename}
    pub async fn save_file(
        &self,
        building_id: Uuid,
        filename: &str,
        content: &[u8],
    ) -> Result<String, String> {
        // Create building-specific directory
        let building_dir = self.base_path.join(building_id.to_string());
        if !building_dir.exists() {
            fs::create_dir_all(&building_dir)
                .map_err(|e| format!("Failed to create building directory: {}", e))?;
        }

        // Generate unique filename to avoid collisions
        let unique_filename = self.generate_unique_filename(filename);
        let file_path = building_dir.join(&unique_filename);

        // Write file to disk
        let mut file =
            fs::File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

        file.write_all(content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Return relative path (from base_path)
        let relative_path = format!("{}/{}", building_id, unique_filename);
        Ok(relative_path)
    }

    /// Read a file from storage
    pub async fn read_file(&self, relative_path: &str) -> Result<Vec<u8>, String> {
        let file_path = self.base_path.join(relative_path);

        if !file_path.exists() {
            return Err("File not found".to_string());
        }

        fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))
    }

    /// Delete a file from storage
    pub async fn delete_file(&self, relative_path: &str) -> Result<(), String> {
        let file_path = self.base_path.join(relative_path);

        if !file_path.exists() {
            return Err("File not found".to_string());
        }

        fs::remove_file(&file_path).map_err(|e| format!("Failed to delete file: {}", e))
    }

    /// Check if a file exists
    pub async fn file_exists(&self, relative_path: &str) -> bool {
        self.base_path.join(relative_path).exists()
    }

    /// Generate a unique filename by prepending UUID to original filename
    fn generate_unique_filename(&self, original: &str) -> String {
        let uuid = Uuid::new_v4();
        format!("{}_{}", uuid, self.sanitize_filename(original))
    }

    /// Sanitize filename to prevent path traversal attacks
    fn sanitize_filename(&self, filename: &str) -> String {
        // Replace path separators and sanitize the filename
        filename.replace("..", "_").replace(['/', '\\'], "_")
    }
}

#[async_trait]
impl StorageProvider for FileStorage {
    async fn save_file(
        &self,
        building_id: Uuid,
        filename: &str,
        content: &[u8],
    ) -> Result<String, String> {
        let start = Instant::now();
        let result = FileStorage::save_file(self, building_id, filename, content).await;
        record_storage_operation(
            "local",
            "save_file",
            start.elapsed(),
            result.as_ref().map(|_| ()).map_err(|e| e.as_str()),
        );
        result
    }

    async fn read_file(&self, relative_path: &str) -> Result<Vec<u8>, String> {
        let start = Instant::now();
        let result = FileStorage::read_file(self, relative_path).await;
        record_storage_operation(
            "local",
            "read_file",
            start.elapsed(),
            result.as_ref().map(|_| ()).map_err(|e| e.as_str()),
        );
        result
    }

    async fn delete_file(&self, relative_path: &str) -> Result<(), String> {
        let start = Instant::now();
        let result = FileStorage::delete_file(self, relative_path).await;
        record_storage_operation(
            "local",
            "delete_file",
            start.elapsed(),
            result.as_ref().map(|_| ()).map_err(|e| e.as_str()),
        );
        result
    }

    async fn file_exists(&self, relative_path: &str) -> bool {
        let start = Instant::now();
        let exists = FileStorage::file_exists(self, relative_path).await;
        record_storage_operation("local", "file_exists", start.elapsed(), Ok(()));
        exists
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_save_and_read_file() {
        let temp_dir = env::temp_dir().join("koprogo_test_storage");
        let storage = FileStorage::new(&temp_dir).unwrap();

        let building_id = Uuid::new_v4();
        let content = b"Test file content";

        let path = storage
            .save_file(building_id, "test.txt", content)
            .await
            .unwrap();

        assert!(storage.file_exists(&path).await);

        let read_content = storage.read_file(&path).await.unwrap();
        assert_eq!(read_content, content);

        // Cleanup
        storage.delete_file(&path).await.unwrap();
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = env::temp_dir().join("koprogo_test_storage_delete");
        let storage = FileStorage::new(&temp_dir).unwrap();

        let building_id = Uuid::new_v4();
        let content = b"Test content";

        let path = storage
            .save_file(building_id, "delete_me.txt", content)
            .await
            .unwrap();

        assert!(storage.file_exists(&path).await);

        storage.delete_file(&path).await.unwrap();
        assert!(!storage.file_exists(&path).await);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_sanitize_filename() {
        let temp_dir = env::temp_dir().join("koprogo_test");
        let storage = FileStorage::new(&temp_dir).unwrap();

        assert_eq!(storage.sanitize_filename("../etc/passwd"), "__etc_passwd");
        assert_eq!(storage.sanitize_filename("normal.pdf"), "normal.pdf");

        fs::remove_dir_all(&temp_dir).ok();
    }
}

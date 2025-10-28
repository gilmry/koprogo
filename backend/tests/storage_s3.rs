use koprogo_api::infrastructure::storage::{S3Storage, S3StorageConfig, StorageProvider};
use serial_test::serial;
use testcontainers_modules::{
    minio::MinIO,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};
use uuid::Uuid;

async fn start_minio() -> ContainerAsync<MinIO> {
    MinIO::default()
        .start()
        .await
        .expect("Failed to start MinIO container")
}

#[tokio::test]
#[serial]
async fn s3_storage_roundtrip() {
    let minio = start_minio().await;
    let api_port = minio
        .get_host_port_ipv4(9000)
        .await
        .expect("Failed to get MinIO port");

    let endpoint = format!("http://127.0.0.1:{api_port}");
    let bucket = format!("koprogo-test-{}", Uuid::new_v4().simple());

    let config = S3StorageConfig {
        bucket: bucket.clone(),
        region: Some("us-east-1".to_string()),
        endpoint: Some(endpoint),
        access_key: "minioadmin".to_string(),
        secret_key: "minioadmin".to_string(),
        force_path_style: true,
        key_prefix: Some("integration-tests".to_string()),
    };

    let storage = S3Storage::from_config(config)
        .await
        .expect("Failed to initialize S3 storage");

    let building_id = Uuid::new_v4();
    let payload = b"Hello KoproGo!";

    let path = storage
        .save_file(building_id, "test.txt", payload)
        .await
        .expect("Save file failed");

    assert!(
        storage.file_exists(&path).await,
        "file should exist immediately after save"
    );

    let downloaded = storage
        .read_file(&path)
        .await
        .expect("Failed to download file");
    assert_eq!(downloaded, payload);

    storage
        .delete_file(&path)
        .await
        .expect("Failed to delete file");

    assert!(
        !storage.file_exists(&path).await,
        "file should not exist after delete"
    );

    // Ensure bucket remains accessible for subsequent operations
    let _ = storage
        .save_file(building_id, "second.txt", b"Second round")
        .await
        .expect("Bucket should still accept uploads");
}

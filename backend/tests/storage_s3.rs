use koprogo_api::infrastructure::storage::{S3Storage, S3StorageConfig, StorageProvider};
use serial_test::serial;
use testcontainers_modules::{
    minio::MinIO,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use uuid::Uuid;

/// Le fork communautaire pgsty (#1036, choix du PO le 2026-10-29), la même
/// image que les compose. Épinglé par tag daté ET digest : le test ne dépend
/// d'aucune publication à venir.
const REGISTRE_MINIO: &str = "pgsty/minio";
const TAG_MINIO: &str = "RELEASE.2026-08-04T00-00-00Z@sha256:b6bfe7239bfc83fb90d31612d9704d86039dd714f7904b3f1ad68f211e602372";

/// Le registre est nommé ICI, et pas laissé au module (#877, puis #1036 :
/// quay.io a fini par refuser à son tour, et MinIO ne publie plus d'images
/// communautaires).
///
/// ── Ce qui s'est passé ────────────────────────────────────────────────────
///
/// `testcontainers_modules::minio` code `minio/minio` en dur (`minio/mod.rs`),
/// c'est-à-dire `docker.io`. Or **tout le dépôt `docker.io/minio/minio` a
/// disparu**, `:latest` compris — mesuré le 2026-09-13 :
///
/// ```text
/// minio/minio:RELEASE.2025-02-28T09-55-16Z           ÉCHEC (404)
/// minio/minio:latest                                 ÉCHEC (404)
/// quay.io/minio/minio:RELEASE.2025-02-28T09-55-16Z   OK
/// ```
///
/// Le diagnostic initial de #877 — « l'éditeur a retiré CE tag » — était donc
/// trop étroit d'un cran : il n'y avait plus de tag à rafraîchir. La même
/// release exacte est servie par quay.io, si bien que ce correctif ne change
/// **aucun bit** : même version, autre registre.
///
/// ── Ce que ce correctif ne fait pas ───────────────────────────────────────
///
/// Il déplace la dépendance, il ne la supprime pas. Un retrait de quay.io
/// réarmerait exactement le même défaut, et le verdict de la CI dépend
/// toujours d'une décision de publication qui n'est pas la nôtre. Le miroir
/// sous notre contrôle reste la seule réponse à la CLASSE de défauts ; il a
/// son issue.
async fn start_minio() -> ContainerAsync<MinIO> {
    MinIO::default()
        .with_name(REGISTRE_MINIO)
        .with_tag(TAG_MINIO)
        .start()
        .await
        .unwrap_or_else(|e| {
            // Nommer l'IMAGE, pas seulement « Failed to start ». Le message
            // précédent laissait un `expect` anonyme sur un `Result` : il
            // fallait lire la trace pour savoir que le tirage avait échoué,
            // et sur quoi.
            panic!("Démarrage de MinIO impossible ({REGISTRE_MINIO}:{TAG_MINIO}) : {e}")
        })
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

use super::{metrics::record_storage_operation, StorageProvider};
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Region};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::env;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Configuration holder for the S3/MinIO storage backend.
#[derive(Clone, Debug)]
pub struct S3StorageConfig {
    pub bucket: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub force_path_style: bool,
    pub key_prefix: Option<String>,
}

impl S3StorageConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, String> {
        let bucket =
            env::var("S3_BUCKET").map_err(|_| "S3_BUCKET is required when using s3 storage")?;
        let access_key = env::var("S3_ACCESS_KEY")
            .map_err(|_| "S3_ACCESS_KEY is required when using s3 storage")?;
        let secret_key = env::var("S3_SECRET_KEY")
            .map_err(|_| "S3_SECRET_KEY is required when using s3 storage")?;

        let region = env::var("S3_REGION").ok();
        let endpoint = env::var("S3_ENDPOINT").ok();
        let key_prefix = env::var("S3_KEY_PREFIX").ok();
        let force_path_style = env::var("S3_FORCE_PATH_STYLE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        Ok(Self {
            bucket,
            region,
            endpoint,
            access_key,
            secret_key,
            force_path_style,
            key_prefix,
        })
    }
}

pub struct S3Storage {
    client: Client,
    bucket: String,
    key_prefix: Option<String>,
}

impl S3Storage {
    /// Build a storage instance from a configuration object.
    pub async fn from_config(config: S3StorageConfig) -> Result<Self, String> {
        let S3StorageConfig {
            bucket,
            region,
            endpoint,
            access_key,
            secret_key,
            force_path_style,
            key_prefix,
        } = config;

        let region_provider = if let Some(ref region) = region {
            RegionProviderChain::first_try(Region::new(region.clone()))
        } else {
            RegionProviderChain::default_provider()
        };

        let credentials = SharedCredentialsProvider::new(Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "koprogo-storage",
        ));

        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        let mut builder = S3ConfigBuilder::from(&shared_config);

        if let Some(region) = region {
            builder = builder.region(Region::new(region));
        }

        if let Some(endpoint) = endpoint {
            builder = builder.endpoint_url(endpoint);
        }

        if force_path_style {
            builder = builder.force_path_style(true);
        }

        let client = Client::from_conf(builder.build());

        Self::ensure_bucket(&client, &bucket).await?;

        Ok(Self {
            client,
            bucket,
            key_prefix,
        })
    }

    fn build_key(&self, building_id: Uuid, original_name: &str) -> String {
        let sanitized = Self::sanitize_filename(original_name);
        let unique = format!("{}_{}", Uuid::new_v4(), sanitized);
        let key = format!("{}/{}", building_id, unique);
        if let Some(prefix) = &self.key_prefix {
            format!("{}/{}", prefix.trim_end_matches('/'), key)
        } else {
            key
        }
    }

    fn sanitize_filename(filename: &str) -> String {
        filename.replace("..", "_").replace(['/', '\\'], "_")
    }
}

impl S3Storage {
    async fn ensure_bucket(client: &Client, bucket: &str) -> Result<(), String> {
        match client.head_bucket().bucket(bucket).send().await {
            Ok(_) => Ok(()),
            Err(SdkError::ServiceError(err)) if err.err().is_not_found() => {
                match client.create_bucket().bucket(bucket).send().await {
                    Ok(_) => Ok(()),
                    Err(SdkError::ServiceError(err))
                        if err.err().is_bucket_already_exists()
                            || err.err().is_bucket_already_owned_by_you() =>
                    {
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to create bucket `{}`: {}", bucket, e)),
                }
            }
            Err(e) => Err(format!("Failed to verify bucket `{}`: {}", bucket, e)),
        }
    }
}

#[async_trait]
impl StorageProvider for S3Storage {
    async fn save_file(
        &self,
        building_id: Uuid,
        filename: &str,
        content: &[u8],
    ) -> Result<String, String> {
        let start = Instant::now();
        let key = self.build_key(building_id, filename);

        let result = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(content.to_vec()))
            .send()
            .await
            .map_err(|e| format!("Failed to upload object: {}", e))
            .map(|_| key.clone());

        record_storage_operation(
            "s3",
            "save_file",
            start.elapsed(),
            result.as_ref().map(|_| ()).map_err(|e| e.as_str()),
        );

        result
    }

    async fn read_file(&self, relative_path: &str) -> Result<Vec<u8>, String> {
        let start = Instant::now();
        let result = match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(relative_path)
            .send()
            .await
        {
            Ok(output) => match output.body.collect().await {
                Ok(data) => Ok(data.into_bytes().to_vec()),
                Err(e) => Err(format!("Failed to read object body: {}", e)),
            },
            Err(e) => Err(format!("Failed to fetch object: {}", e)),
        };

        record_storage_operation(
            "s3",
            "read_file",
            start.elapsed(),
            result.as_ref().map(|_| ()).map_err(|e| e.as_str()),
        );

        result
    }

    async fn delete_file(&self, relative_path: &str) -> Result<(), String> {
        let start = Instant::now();
        let result = self
            .client
            .delete_object()
            .bucket(&self.bucket)
            .key(relative_path)
            .send()
            .await
            .map_err(|e| format!("Failed to delete object: {}", e))
            .map(|_| ());

        record_storage_operation(
            "s3",
            "delete_file",
            start.elapsed(),
            result.as_ref().map(|_| ()).map_err(|e| e.as_str()),
        );

        result
    }

    async fn file_exists(&self, relative_path: &str) -> bool {
        let start = Instant::now();
        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(relative_path)
            .send()
            .await;

        let exists = match result {
            Ok(_) => true,
            Err(SdkError::ServiceError(err)) if err.err().is_not_found() => false,
            Err(_) => false,
        };

        record_storage_operation("s3", "file_exists", start.elapsed(), Ok(()));

        exists
    }
}

impl std::fmt::Debug for S3Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Storage")
            .field("bucket", &self.bucket)
            .field("key_prefix", &self.key_prefix)
            .finish()
    }
}

/// Convenient alias for sharing the S3 storage provider.
pub type SharedS3Storage = Arc<S3Storage>;

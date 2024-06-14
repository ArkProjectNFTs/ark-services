use anyhow::Result;
use arkproject::metadata::file_manager::{FileInfo, FileManager};
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::primitives::ByteStream;
use mime_guess::from_path;
use tracing::{error, info};

/// An implementation of the FileManager trait that utilizes AWS S3 for storage.
/// Requires a specified bucket name to interface with AWS S3.
#[derive(Default)]
pub struct AWSFileManager {
    bucket_name: String,
}

#[async_trait]
impl FileManager for AWSFileManager {
    /// Saves a file to AWS S3 and returns the URL to the uploaded file.
    /// If the file already exists (determined by the hash of the contents),
    /// it returns the URL to the existing file instead of re-uploading it.
    async fn save(&self, file: &FileInfo) -> Result<String> {
        info!("Checking if '{}' exists on AWS S3...", file.name);

        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = aws_sdk_s3::Client::new(&config);

        // Compute the SHA-256 hash of the file content to use as the file key.
        let hash = sha256::digest(&file.content);
        let file_extension = file.name.split('.').last().unwrap_or("");
        let key = file.dir_path.as_ref().map_or_else(
            || format!("{}.{}", hash, file_extension),
            |dir_path| format!("{}/{}.{}", dir_path, hash, file_extension),
        );

        let content_type = from_path(&file.name)
            .first_or_octet_stream()
            .as_ref()
            .to_string();

        // Check if the file already exists on S3 using a head_object request.
        if client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .is_ok()
        {
            info!("File '{}' already exists on AWS S3: {}", file.name, key);
            return Ok(key);
        }

        // If the file does not exist, proceed to upload.
        let body = ByteStream::from(file.content.clone());
        match client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .content_disposition("inline")
            .content_type(content_type)
            .send()
            .await
        {
            Ok(_) => {
                info!("Uploaded '{}' to AWS S3: {}", file.name, key);
                Ok(key)
            }
            Err(e) => {
                // On failure, log the error and return it.
                error!("Failed to upload '{}' to AWS S3: {}", file.name, e);
                Err(anyhow::anyhow!(
                    "Failed to upload '{}' to AWS S3: {}",
                    file.name,
                    e
                ))
            }
        }
    }
}

impl AWSFileManager {
    pub fn new(bucket_name: String) -> Self {
        Self { bucket_name }
    }
}

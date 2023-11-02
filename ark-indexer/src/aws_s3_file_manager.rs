use anyhow::Result;
use arkproject::metadata::file_manager::{FileInfo, FileManager};
use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use tracing::{debug, error, info};

/// FileManager implementation that saves files to AWS S3.
///
/// This implementation requires a bucket name for storing files in AWS S3.
#[derive(Default)]
pub struct AWSFileManager {
    bucket_name: String,
}

// TODO: remove this once used.
#[allow(dead_code)]
impl AWSFileManager {
    /// Create a new AWSFileManager with the specified bucket name.
    pub fn new(bucket_name: String) -> Self {
        Self { bucket_name }
    }
}

#[async_trait]
impl FileManager for AWSFileManager {
    async fn save(&self, file: &FileInfo) -> Result<()> {
        info!(
            "Starting upload of '{}' to AWS S3 bucket '{}'.",
            file.name, &self.bucket_name
        );

        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);
        let body = ByteStream::from(file.content.clone());
        let key = match &file.dir_path {
            Some(dir_path) => {
                debug!(
                    "Resolved directory path for '{}': '{}'.",
                    file.name, dir_path
                );
                format!("{}/{}", dir_path, &file.name)
            }
            None => {
                debug!(
                    "No directory path provided for '{}'. Using filename as key.",
                    file.name
                );
                file.name.clone()
            }
        };

        debug!(
            "Preparing to upload to Bucket='{}', Key='{}'.",
            &self.bucket_name, &key
        );

        match client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .send()
            .await
        {
            Ok(_) => {
                info!(
                    "Successfully uploaded '{}' to AWS S3 bucket '{}'.",
                    file.name, &self.bucket_name
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Upload failure for '{}' to AWS S3 bucket '{}': {}",
                    file.name, &self.bucket_name, e
                );
                Err(anyhow::anyhow!(
                    "Failed to upload '{}' to AWS S3 bucket '{}': {}",
                    file.name,
                    &self.bucket_name,
                    e
                ))
            }
        }
    }
}

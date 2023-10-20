use anyhow::Result;
use arkproject::metadata::file_manager::{FileInfo, FileManager};
use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use tracing::{info, error, debug};

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
        info!("Uploading {} to AWS...", file.name);

        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);
        let body = ByteStream::from(file.content.clone());
        let key = match &file.dir_path {
            Some(dir_path) => format!("{}/{}", dir_path, &file.name),
            None => file.name.clone(),
        };

        debug!("Bucket={}, Key={}", &self.bucket_name, key);

        match client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(body)
            .send()
            .await
        {
            Ok(_) => {
                debug!("Uploaded {} to AWS", file.name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to upload {} to AWS: {}", file.name, e);
                return Err(anyhow::anyhow!(
                    "Failed to upload {} to AWS: {}",
                    file.name,
                    e
                ))
            }
        }
    }
}

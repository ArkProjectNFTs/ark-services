use anyhow::Result;
use arkproject::metadata::file_manager::{FileInfo, FileManager};
use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
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

        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);

        // Compute the SHA-256 hash of the file content to use as the file key.
        let hash = sha256::digest(&file.content);
        let file_extension = file.name.split('.').last().unwrap_or("");
        let key = file.dir_path.as_ref().map_or_else(
            || format!("{}.{}", hash, file_extension),
            |dir_path| format!("{}/{}.{}", dir_path, hash, file_extension),
        );

        // Check if the file already exists on S3 using a head_object request.
        if client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .is_ok()
        {
            // If the file exists, construct the URL and return it.
            let url = format!("https://{}.s3.amazonaws.com/{}", &self.bucket_name, key);
            info!("File '{}' already exists on AWS S3: {}", file.name, url);
            return Ok(url);
        }

        // If the file does not exist, proceed to upload.
        let body = ByteStream::from(file.content.clone());
        match client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .send()
            .await
        {
            Ok(_) => {
                // On successful upload, construct the file URL and return it.
                let url = format!("https://{}.s3.amazonaws.com/{}", &self.bucket_name, key);
                info!("Uploaded '{}' to AWS S3: {}", file.name, url);
                Ok(url)
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

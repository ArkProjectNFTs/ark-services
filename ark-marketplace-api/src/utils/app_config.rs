use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub port: u64,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_default_region: String,
    pub rust_log: String,
    pub aws_secret_read_db: String,
    pub aws_secret_write_db: String,
    pub aws_secret_redis_db: String,
    pub aws_secret_eleasticsearch_db: String,
}

impl AppConfig {
    pub fn load_from_file(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let builder = Config::builder();
        let settings = builder
            .add_source(File::new(config_path, FileFormat::Yaml))
            .build()?;
        let config: AppConfig = settings.try_deserialize()?;

        Ok(config)
    }
}

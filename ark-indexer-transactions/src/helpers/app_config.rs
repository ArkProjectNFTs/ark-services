use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub log_level: String,
    pub rcp_provider: String,
    pub base_path: String,
    pub parsing_state_path: String,
    pub chain_id: String,
    pub start_from: u64,
    pub orderbooks: Vec<String>,
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

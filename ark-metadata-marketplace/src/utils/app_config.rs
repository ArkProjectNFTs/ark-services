use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_default_region: String,
    pub aws_secret_bucket_name: String,
    pub rcp_provider: String,
    pub chain_id: String,
    pub ipfs_timeout_duration: String,
    pub loop_delay_duration: String,
    pub ipfs_gateway_uri: String,
    pub filter: Option<String>,
    pub refresh_contract_metadata: bool,
    pub rust_log: String,
    pub aws_secret_read_db: String,
    pub aws_secret_eleasticsearch_db: String,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_default_region: String,
    pub aws_secret_bucket_name: String,
    pub rcp_provider: String,
    pub chain_id: String,
    pub ipfs_timeout_duration: String,
    pub loop_delay_duration: String,
    pub ipfs_gateway_uri: String,
    pub filter: Option<(String, String)>,
    pub refresh_contract_metadata: bool,
    pub rust_log: String,
    pub aws_secret_read_db: String,
    pub aws_secret_eleasticsearch_db: String,
}

impl OutputConfig {
    pub fn load_from_file(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let builder = Config::builder();
        let settings = builder
            .add_source(File::new(config_path, FileFormat::Yaml))
            .build()?;
        let config: AppConfig = settings.try_deserialize()?;
        let mut out_config = OutputConfig {
            aws_access_key_id: config.aws_access_key_id,
            aws_secret_access_key: config.aws_secret_access_key,
            aws_default_region: config.aws_default_region,
            aws_secret_bucket_name: config.aws_secret_bucket_name,
            rcp_provider: config.rcp_provider,
            chain_id: config.chain_id.clone(),
            ipfs_timeout_duration: config.ipfs_timeout_duration,
            loop_delay_duration: config.loop_delay_duration,
            ipfs_gateway_uri: config.ipfs_gateway_uri,
            filter: None,
            refresh_contract_metadata: config.refresh_contract_metadata,
            rust_log: config.rust_log,
            aws_secret_eleasticsearch_db: config.aws_secret_eleasticsearch_db,
            aws_secret_read_db: config.aws_secret_read_db,
        };
        if let Some(filter) = config.filter {
            out_config.filter = Some((filter, config.chain_id));
        }

        Ok(out_config)
    }
}

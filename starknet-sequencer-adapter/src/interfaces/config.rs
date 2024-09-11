use crate::helpers::config::{
    default_blocks_per_file, default_max_call_per_minute, default_monitor_threads,
    default_progress_bar_width, default_threads,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_max_call_per_minute")]
    pub max_calls_per_minute: u32,
    #[serde(default = "default_threads")]
    pub threads: usize,
    #[serde(default = "default_monitor_threads")]
    pub monitor_threads: usize,
    #[serde(default = "default_blocks_per_file")]
    pub blocks_per_file: u64,
    #[serde(default = "default_progress_bar_width")]
    pub progress_bar_width: usize,
}

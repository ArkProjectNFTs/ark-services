// DEFINE THE DEFAULT CONFIGURATION WHEN NO ENV FILE IS PASSED
const MAX_CALLS_PER_MINUTE: u32 = 180;
const THREADS: usize = 1;
const MONITOR_THREADS: usize = 1;
const BLOCKS_PER_FILE: u64 = 100;
const PROGRESS_BAR_WIDTH: usize = 50;

const STORAGE_ROOT_DIR: &str = "/opt/fast-indexer";

pub fn default_max_call_per_minute() -> u32 {
    MAX_CALLS_PER_MINUTE
}

pub fn default_threads() -> usize {
    THREADS
}

pub fn default_monitor_threads() -> usize {
    MONITOR_THREADS
}

pub fn default_blocks_per_file() -> u64 {
    BLOCKS_PER_FILE
}

pub fn default_progress_bar_width() -> usize {
    PROGRESS_BAR_WIDTH
}

pub fn default_storage_dir() -> String {
    STORAGE_ROOT_DIR.to_owned()
}

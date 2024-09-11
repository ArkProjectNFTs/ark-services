use std::io::Write;
use tokio::time::Instant;

pub fn update_progress(
    width: usize,
    processed_blocks: usize,
    total_blocks: u64,
    start_time: Instant,
    last_update: Instant,
    blocks_last_minute: usize,
) -> Instant {
    let percentage = (processed_blocks as f64 / total_blocks as f64) * 100.0;
    let elapsed = start_time.elapsed().as_secs();
    let eta = if processed_blocks > 0 {
        let avg_time_per_block = elapsed as f64 / processed_blocks as f64;
        println!("processed: {}", processed_blocks);
        let remaining_blocks = total_blocks as usize - processed_blocks;
        avg_time_per_block * remaining_blocks as f64
    } else {
        0.0
    };

    let elapsed_since_last_update = last_update.elapsed().as_secs();
    let speed = if elapsed_since_last_update > 0 {
        (blocks_last_minute as f64 / elapsed_since_last_update as f64) * 60.0
    } else {
        0.0
    };

    let progress_bar_filled_length =
        (width as f64 * (processed_blocks as f64 / total_blocks as f64)).round() as usize;
    let progress_bar: String =
        "=".repeat(progress_bar_filled_length) + &" ".repeat(width - progress_bar_filled_length);

    print!(
        "\r[{}] {}/{} [{:.2}%] - ETA: {:.2}s - Speed: {:.2} blocks/min",
        progress_bar, processed_blocks, total_blocks, percentage, eta, speed
    );
    std::io::stdout().flush().unwrap();

    Instant::now()
}

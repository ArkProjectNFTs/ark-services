use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const BLOCKS_PER_FILE: usize = 10_000;

pub struct StateManager {
    base_path: PathBuf,
    cache: Arc<Mutex<HashMap<usize, bool>>>,
}

impl StateManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> io::Result<Self> {
        fs::create_dir_all(&base_path)?;
        Ok(Self {
            base_path: base_path.as_ref().to_path_buf(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_file_path(&self, block_number: usize) -> PathBuf {
        let file_number = block_number / BLOCKS_PER_FILE;
        self.base_path.join(format!("state_{}.bin", file_number))
    }

    pub fn set_block_state(&self, block_number: usize, state: bool) -> io::Result<()> {
        let file_path = self.get_file_path(block_number);
        let offset = (block_number % BLOCKS_PER_FILE) / 8;
        let bit_position = (block_number % BLOCKS_PER_FILE) % 8;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&file_path)?;

        let file_size = file.metadata()?.len();
        if file_size <= offset as u64 {
            file.set_len((offset + 1) as u64)?;
        }

        let mut buffer = [0u8; 1];
        file.seek(SeekFrom::Start(offset as u64))?;
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            buffer[0] = 0;
        }

        if state {
            buffer[0] |= 1 << bit_position;
        } else {
            buffer[0] &= !(1 << bit_position);
        }

        file.seek(SeekFrom::Start(offset as u64))?;
        file.write_all(&buffer)?;

        let mut cache = self.cache.lock().unwrap();
        cache.insert(block_number, state);

        Ok(())
    }

    pub fn get_block_state(&self, block_number: usize) -> io::Result<bool> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(&state) = cache.get(&block_number) {
                return Ok(state);
            }
        }

        let file_path = self.get_file_path(block_number);
        let offset = (block_number % BLOCKS_PER_FILE) / 8;
        let bit_position = (block_number % BLOCKS_PER_FILE) % 8;

        let mut file = File::open(&file_path)?;
        let file_size = file.metadata()?.len();

        if file_size <= offset as u64 {
            return Ok(false); // Le bloc n'a pas encore été traité
        }

        let mut buffer = [0u8; 1];
        file.seek(SeekFrom::Start(offset as u64))?;
        let bytes_read = file.read(&mut buffer)?;

        let state = if bytes_read == 0 {
            false
        } else {
            (buffer[0] & (1 << bit_position)) != 0
        };

        let mut cache = self.cache.lock().unwrap();
        cache.insert(block_number, state);

        Ok(state)
    }

    pub fn get_last_processed_block(&self) -> io::Result<usize> {
        let mut last_processed = 0;
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            if file_name.starts_with("state_") && file_name.ends_with(".bin") {
                let file_number: usize = file_name[6..file_name.len() - 4].parse().unwrap();
                let file_path = self.base_path.join(file_name);
                let file = File::open(file_path)?;
                let mut reader = BufReader::new(file);
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)?;

                for (i, &byte) in buffer.iter().enumerate().rev() {
                    if byte != 0 {
                        for j in (0..8).rev() {
                            if byte & (1 << j) != 0 {
                                return Ok(file_number * BLOCKS_PER_FILE + i * 8 + j);
                            }
                        }
                    }
                }

                last_processed = file_number * BLOCKS_PER_FILE + buffer.len() * 8 - 1;
            }
        }
        Ok(last_processed)
    }
}

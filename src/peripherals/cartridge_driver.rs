use std::fs::File;
use std::io::prelude::*;

use crate::constants::MAX_ROM_SIZE;

pub(crate) struct CartridgeDriver {
    pub rom: [u8; MAX_ROM_SIZE],
    pub rom_size: usize,
}

impl CartridgeDriver {
    pub(crate) fn new(filename: &str) -> Self {
        let mut f = File::open(filename).expect("ERROR: file not found");
        let mut buffer = [0u8; MAX_ROM_SIZE];

        let bytes_read = if let Ok(bytes_read) = f.read(&mut buffer) {
            bytes_read
        } else {
            0
        };

        CartridgeDriver {
            rom: buffer,
            rom_size: bytes_read,
        }
    }
}

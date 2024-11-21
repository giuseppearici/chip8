use log::error;

use crate::constants::{MAX_ROM_SIZE, MEMORY_SIZE, RESERVED_MEMORY_SIZE};

//  Memory Map:
//  +---------------+= 0xFFF (4095) End of Chip-8 RAM
//  |               |
//  |               |
//  |               |
//  |               |
//  |               |
//  | 0x200 to 0xFFF|
//  |     Chip-8    |
//  | Program / Data|
//  |     Space     |
//  |               |
//  |               |
//  |               |
//  +- - - - - - - -+= 0x600 (1536) Start of ETI 660 Chip-8 programs
//  |               |
//  |               |
//  |               |
//  +---------------+= 0x200 (512) Start of most Chip-8 programs
//  | 0x000 to 0x1FF|
//  | Reserved for  |
//  |  interpreter  |
//  +---------------+= 0x000 (0) Start of Chip-8 RAM

pub(crate) struct Memory {
    bytes: [u8; MEMORY_SIZE],
    rom_size: usize,
}

impl Memory {
    pub(crate) fn new() -> Self {
        let mut bytes = [0u8; MEMORY_SIZE];
        for i in 0..FONT_SPRITES.len() {
            bytes[i] = FONT_SPRITES[i];
        }
        Memory { bytes, rom_size: 0 }
    }

    pub(crate) fn reset(&mut self, rom_bytes: &[u8], rom_size: usize) {
        if rom_size > MAX_ROM_SIZE {
            error!("ROM size exceeds memory capacity");
        }
        self.rom_size = rom_size;
        for (i, &byte) in rom_bytes.iter().enumerate() {
            let address = RESERVED_MEMORY_SIZE + i;
            if address < RESERVED_MEMORY_SIZE + rom_size {
                self.bytes[RESERVED_MEMORY_SIZE + i] = byte;
            } else {
                break;
            }
        }
    }

    pub(crate) fn load(&self, address: usize) -> u8 {
        self.bytes[address]
    }

    pub(crate) fn store(&mut self, address: usize, value: u8) {
        self.bytes[address] = value;
    }
}

pub const FONT_SPRITES: [u8; 5 * 16] = [
    // 0
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1001_0000, // | *  * |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    // 1
    0b_0010_0000, // |   *  |
    0b_0110_0000, // |  **  |
    0b_0010_0000, // |   *  |
    0b_0010_0000, // |   *  |
    0b_0111_0000, // |  *** |
    // 2
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    // 3
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_1111_0000, // | **** |
    // 4
    0b_1001_0000, // | *  * |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_0001_0000, // |    * |
    // 5
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_1111_0000, // | **** |
    // 6
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    // 7
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_0010_0000, // |   *  |
    0b_0100_0000, // |  *   |
    0b_0100_0000, // |  *   |
    // 8
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    // 9
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    0b_0001_0000, // |    * |
    0b_1111_0000, // | **** |
    // A
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1111_0000, // | **** |
    0b_1001_0000, // | *  * |
    0b_1001_0000, // | *  * |
    // B
    0b_1110_0000, // | ***  |
    0b_1001_0000, // | *  * |
    0b_1110_0000, // | ***  |
    0b_1001_0000, // | *  * |
    0b_1110_0000, // | ***  |
    // C
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1000_0000, // | *    |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    // D
    0b_1110_0000, // | ***  |
    0b_1001_0000, // | *  * |
    0b_1001_0000, // | *  * |
    0b_1001_0000, // | *  * |
    0b_1110_0000, // | ***  |
    // E
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    // F
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1111_0000, // | **** |
    0b_1000_0000, // | *    |
    0b_1000_0000, // | *    |
];

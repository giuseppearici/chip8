pub const SCREEN_WIDTH: usize = 64; // The width of the CHIP-8 display in pixels (64 pixels)
pub const SCREEN_HEIGHT: usize = 32; // The height of the CHIP-8 display in pixels (32 pixels)
pub const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT; // The total number of pixels in the CHIP-8 display (2048 pixels)
pub const SCALE_FACTOR: u32 = 20; // The scaling factor for rendering the CHIP-8 display (useful for modern screens)

pub const FOREGROUND_COLOR: [u8; 3] = [65, 236, 157]; // RGB color used for the foreground (active pixels) on the display
pub const BACKGROUND_COLOR: [u8; 3] = [15, 15, 15]; // RGB color used for the background (inactive pixels) on the display

pub const MEMORY_SIZE: usize = 4096; // Total memory size for the CHIP-8 system (4KB), typical of the CHIP-8 architecture
pub const RESERVED_MEMORY_SIZE: usize = 512; // Reserved memory space (0x000 to 0x1FF) for interpreter, font data, and other purposes
pub const MAX_ROM_SIZE: usize = MEMORY_SIZE - RESERVED_MEMORY_SIZE;

pub const FRAME_FREQUENCY: f64 = 60.0; // Target frame rate for the CHIP-8 system (60 frames per second)
pub const FRAME_SIZE: usize = 15; // Number of CPU cycles (instructions) to execute per frame

pub const OPCODE_SIZE: usize = 2; // Size of each opcode in bytes (2 bytes per instruction in CHIP-8)

pub const V_REGISTERS_SIZE: usize = 16; // Size of general-purpose registers in the CHIP-8 system (V0 to VF)
pub const STACK_SIZE: usize = 16; // Size of the stack used for subroutine calls and returns

pub const LOG_FILE_PATH: &str = "debug.log"; // Path to the log file for storing debug information
pub const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug; // Default log level for the CHIP-8 emulator

pub const SEGMENTS_AFTER_PROGRAM_COUNTER: usize = 10; // Number of segments after the program counter to display in the disassembly view

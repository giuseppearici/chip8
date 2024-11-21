use rand::Rng;
use std::thread;
use std::time::Duration;

use crate::constants::{
    FRAME_FREQUENCY, FRAME_SIZE, OPCODE_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, STACK_SIZE,
    V_REGISTERS_SIZE,
};
use crate::peripherals::audio_driver::AudioDriver;
use crate::peripherals::cartridge_driver::CartridgeDriver;
use crate::peripherals::display_driver::DisplayDriver;
use crate::peripherals::input_driver::InputDriver;
use crate::toolchain::debugger::Debugger;
use crate::toolchain::decoder::DecodedOpcode;

use super::memory::Memory;
use super::screen::Screen;

/// Represents the CHIP-8 processor, handling memory, registers, stack, and timers
pub(crate) struct Processor {
    /// CHIP-8 memory, array of 4096 bytes
    pub(crate) memory: Memory,

    /// Screen for rendering the 64x32 pixel display
    pub(crate) screen: Screen,

    /// Stack for subroutine calls, 16 entries, each storing a memory index
    pub(crate) stack: [usize; STACK_SIZE],

    /// Stack pointer storing a stack index for the next subroutine call
    pub(crate) stack_pointer: usize,

    /// General-purpose registers, 16 registers, 8-bit each
    pub(crate) v_registers: [u8; V_REGISTERS_SIZE],

    /// Special register storing a memory index for various operations
    pub(crate) i_register: usize,

    /// Program counter storing a memory index for the next instruction
    pub(crate) program_counter: usize,

    /// Delay timer, 8 bits (decremented at 60Hz if non-zero)
    pub(crate) delay_timer: u8,

    /// Sound timer, 8 bits (decremented at 60Hz if non-zero, sound is played if non-zero)
    pub(crate) sound_timer: u8,

    /// Current state of the keypad, 16 keys, 1 bit per key (0 = not pressed, 1 = pressed)
    pub(crate) keypad: u16,

    /// Flag indicating waiting for a keypress, 1 bit (true = waiting, false = not waiting)
    keypad_wait: bool,

    /// Keypad index for keypress result (0-15)
    keypad_wait_index: usize,

    /// Debugger for debugging the processor
    debugger: Debugger,
}

impl Processor {
    pub(crate) fn new() -> Self {
        Processor {
            memory: Memory::new(),
            screen: Screen::new(),
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            v_registers: [0; V_REGISTERS_SIZE],
            i_register: 0,
            program_counter: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            keypad: 0,
            keypad_wait: false,
            keypad_wait_index: 0,
            debugger: Debugger::new(),
        }
    }

    pub(crate) fn run(
        &mut self,
        display_driver: &mut DisplayDriver,
        input_driver: &mut InputDriver,
        audio_driver: &AudioDriver,
        cartridge_driver: &CartridgeDriver,
    ) {
        self.memory
            .reset(&cartridge_driver.rom, cartridge_driver.rom_size);
        self.debugger
            .reset(&cartridge_driver.rom, cartridge_driver.rom_size);

        // Debug the rom only if the DEBUG environment variable is set
        self.debug_rom();

        let sleep_duration = Duration::from_nanos((1f64 / FRAME_FREQUENCY * 1e9) as u64);
        let mut opcode_count = 0;

        while let Ok(keypad) = input_driver.poll() {
            self.tick(keypad);
            opcode_count += 1;

            if self.sound_timer > 0 {
                audio_driver.start_beep();
            } else {
                audio_driver.stop_beep();
            }

            self.screen.refresh(display_driver);

            // Opcode buffer for 60Hz, ideal around 10-15: 60 * 15 = 900 cycles/second
            if opcode_count >= FRAME_SIZE {
                opcode_count = 0;
                if self.sound_timer > 0 {
                    self.sound_timer -= 1;
                }
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
                thread::sleep(sleep_duration);
            }
        }
    }

    fn tick(&mut self, keypad: u16) {
        self.keypad = keypad;

        if self.keypad_wait {
            if self.keypad > 0 {
                self.keypad_wait = false;
                self.v_registers[self.keypad_wait_index] = self.keypad.trailing_zeros() as u8;
            }
        } else {
            // Do the fetch-decode-execute cycle
            let address = self.program_counter;
            let opcode = self.fetch_opcode(address);
            let decoded = self.decode_opcode(opcode);

            // Debug the processor status only if the DEBUG environment variable is set
            self.debug_status(address, opcode, &decoded);

            self.execute_opcode(decoded);
        }
    }

    pub(crate) fn fetch_opcode(&self, address: usize) -> u16 {
        (self.memory.load(address) as u16) << 8 | (self.memory.load(address + 1) as u16)
    }

    pub(crate) fn decode_opcode(&self, opcode: u16) -> DecodedOpcode {
        DecodedOpcode::new(opcode)
    }

    pub(crate) fn execute_opcode(&mut self, decoded: DecodedOpcode) {
        let processor_cycle = match decoded {
            DecodedOpcode::Cls => self.execute_cls(),
            DecodedOpcode::Ret => self.execute_ret(),
            DecodedOpcode::SysNnn { nnn } => self.execute_sys_nnn(nnn),
            DecodedOpcode::JpNnn { nnn } => self.execute_jp_nnn(nnn),
            DecodedOpcode::CallNnn { nnn } => self.execute_call_nnn(nnn),
            DecodedOpcode::SeVxNn { vx, nn } => self.execute_se_vx_nn(vx, nn),
            DecodedOpcode::SneVxNn { vx, nn } => self.execute_sne_vx_nn(vx, nn),
            DecodedOpcode::SeVxVy { vx, vy } => self.execute_se_vx_vy(vx, vy),
            DecodedOpcode::LdVxNn { vx, nn } => self.execute_ld_vx_nn(vx, nn),
            DecodedOpcode::AddVxNn { vx, nn } => self.execute_add_vx_nn(vx, nn),
            DecodedOpcode::LdVxVy { vx, vy } => self.execute_ld_vx_vy(vx, vy),
            DecodedOpcode::OrVxVy { vx, vy } => self.execute_ox_vx_vy(vx, vy),
            DecodedOpcode::AndVxVy { vx, vy } => self.execute_and_vx_vy(vx, vy),
            DecodedOpcode::XorVxVy { vx, vy } => self.execute_xor_vx_vy(vx, vy),
            DecodedOpcode::AddVxVy { vx, vy } => self.execute_add_vx_vy(vx, vy),
            DecodedOpcode::SubVxVy { vx, vy } => self.execute_sub_vx_vy(vx, vy),
            DecodedOpcode::ShrVx { vx } => self.execute_shr_vx(vx),
            DecodedOpcode::SubnVxVy { vx, vy } => self.execute_subn_vx_vy(vx, vy),
            DecodedOpcode::ShlVx { vx } => self.execute_shl_vx(vx),
            DecodedOpcode::SneVxVy { vx, vy } => self.execute_sne_vx_vy(vx, vy),
            DecodedOpcode::LdINnn { nnn } => self.execute_ld_i_nnn(nnn),
            DecodedOpcode::JpV0Nnn { nnn } => self.execute_jp_v0_nnn(nnn),
            DecodedOpcode::RndVxNn { vx, nn } => self.execute_rnd_vx_nn(vx, nn),
            DecodedOpcode::DrwVxVyN { vx, vy, n } => self.execute_drw_vx_vy_n(vx, vy, n),
            DecodedOpcode::SkpVx { vx } => self.execute_skp_vx(vx),
            DecodedOpcode::SknpVx { vx } => self.execute_sknp_vx(vx),
            DecodedOpcode::LdVxK { vx } => self.execute_ld_vx_k(vx),
            DecodedOpcode::LdVxDt { vx } => self.execute_ld_vx_dt(vx),
            DecodedOpcode::LdDtVx { vx } => self.execute_ld_dt_vx(vx),
            DecodedOpcode::LdStVx { vx } => self.execute_ld_st_vx(vx),
            DecodedOpcode::LdFVx { vx } => self.execute_ld_f_vx(vx),
            DecodedOpcode::AddIVx { vx } => self.execute_add_i_vx(vx),
            DecodedOpcode::LdAtIVx { vx } => self.execute_ld_at_i_vx(vx),
            DecodedOpcode::LdVxAtI { vx } => self.execute_ld_vx_at_i(vx),
            DecodedOpcode::BcdVx { vx } => self.execute_bcd_vx(vx),
            DecodedOpcode::Unknown { opcode } => self.execute_unknown(opcode),
        };

        match processor_cycle {
            ProcessorCycle::Error(decoded_opcode, message) => {
                eprintln!("ERROR {}: {}", message, decoded_opcode.to_string());
                self.program_counter += OPCODE_SIZE;
            }
            ProcessorCycle::Next => self.program_counter += OPCODE_SIZE,
            ProcessorCycle::Skip => self.program_counter += 2 * OPCODE_SIZE,
            ProcessorCycle::Jump(addr) => self.program_counter = addr,
        }
    }

    // CLS
    // Clear the display.
    fn execute_cls(&mut self) -> ProcessorCycle {
        self.screen.clear();
        ProcessorCycle::Next
    }

    // RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the
    // top of the stack, then subtracts 1 from the stack pointer.
    fn execute_ret(&mut self) -> ProcessorCycle {
        self.stack_pointer -= 1;
        ProcessorCycle::Jump(self.stack[self.stack_pointer])
    }

    // SYS nnn
    // Jump to a machine code routine at nnn.
    fn execute_sys_nnn(&mut self, nnn: usize) -> ProcessorCycle {
        ProcessorCycle::Error(
            DecodedOpcode::SysNnn { nnn },
            "not implemented opcode".to_string(),
        )
    }

    // JP nnn
    // The interpreter sets the program counter to nnn.
    fn execute_jp_nnn(&mut self, nnn: usize) -> ProcessorCycle {
        ProcessorCycle::Jump(nnn)
    }

    // CALL nnn
    // The interpreter increments the stack pointer, then puts the
    // current PC on the top of the stack. The PC is then set to nnn.
    fn execute_call_nnn(&mut self, nnn: usize) -> ProcessorCycle {
        self.stack[self.stack_pointer] = self.program_counter + OPCODE_SIZE;
        self.stack_pointer += 1;
        ProcessorCycle::Jump(nnn)
    }

    // SE Vx, nn
    // Skip next instruction if Vx = nn.
    fn execute_se_vx_nn(&mut self, x: usize, nn: u8) -> ProcessorCycle {
        ProcessorCycle::skip_if(self.v_registers[x] == nn)
    }

    // SNE Vx, nn
    // Skip next instruction if Vx != nn.
    fn execute_sne_vx_nn(&mut self, x: usize, nn: u8) -> ProcessorCycle {
        ProcessorCycle::skip_if(self.v_registers[x] != nn)
    }

    // SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    fn execute_se_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        ProcessorCycle::skip_if(self.v_registers[x] == self.v_registers[y])
    }

    // LD Vx, nn
    // Set Vx = nn.
    fn execute_ld_vx_nn(&mut self, x: usize, nn: u8) -> ProcessorCycle {
        self.v_registers[x] = nn;
        ProcessorCycle::Next
    }

    // ADD Vx, nn
    // Set Vx = Vx + nn.
    fn execute_add_vx_nn(&mut self, x: usize, nn: u8) -> ProcessorCycle {
        let vx = self.v_registers[x] as u16;
        let val = nn as u16;
        let result = vx + val;
        self.v_registers[x] = result as u8;
        ProcessorCycle::Next
    }

    // LD Vx, Vy
    // Set Vx = Vy.
    fn execute_ld_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        self.v_registers[x] = self.v_registers[y];
        ProcessorCycle::Next
    }

    // OR Vx, Vy
    // Set Vx = Vx OR Vy.
    fn execute_ox_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        self.v_registers[x] |= self.v_registers[y];
        ProcessorCycle::Next
    }

    // AND Vx, Vy
    // Set Vx = Vx AND Vy.
    fn execute_and_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        self.v_registers[x] &= self.v_registers[y];
        ProcessorCycle::Next
    }

    // XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    fn execute_xor_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        self.v_registers[x] ^= self.v_registers[y];
        ProcessorCycle::Next
    }

    // ADD Vx, Vy
    // The values of Vx and Vy are added together. If the result is
    // greater than 8 bits (i.e. > 255) VF is set to 1, otherwise 0.
    // Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn execute_add_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        let vx = self.v_registers[x] as u16;
        let vy = self.v_registers[y] as u16;
        let result = vx + vy;
        self.v_registers[x] = result as u8;
        self.v_registers[0x0f] = if result > 0xFF { 1 } else { 0 };
        ProcessorCycle::Next
    }

    // SUB Vx, Vy
    // If Vx > Vy, then VF is set to 1, otherwise 0. T
    // hen Vy is subtracted from Vx, and the results stored in Vx.
    fn execute_sub_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        let temp = if self.v_registers[x] >= self.v_registers[y] {
            1
        } else {
            0
        };
        self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
        self.v_registers[0xf] = temp;
        ProcessorCycle::Next
    }

    // SHR Vx
    // If the least-significant bit of Vx is 1, then VF is set to 1,
    // otherwise 0. Then Vx is divided by 2.
    fn execute_shr_vx(&mut self, x: usize) -> ProcessorCycle {
        let temp = self.v_registers[x] & 1;
        self.v_registers[x] >>= 1;
        self.v_registers[0xf] = temp;
        ProcessorCycle::Next
    }

    // SUBN Vx, Vy
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted
    // from Vy, and the results stored in Vx.
    fn execute_subn_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        let temp = if self.v_registers[y] >= self.v_registers[x] {
            1
        } else {
            0
        };
        self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
        self.v_registers[0xf] = temp;
        ProcessorCycle::Next
    }

    // SHL Vx
    // If the most-significant bit of Vx is 1, then VF is set to 1,
    // otherwise to 0. Then Vx is multiplied by 2.
    fn execute_shl_vx(&mut self, x: usize) -> ProcessorCycle {
        let temp = self.v_registers[x] >> 7;
        self.v_registers[x] <<= 1;
        self.v_registers[0xf] = temp;
        ProcessorCycle::Next
    }

    // SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    fn execute_sne_vx_vy(&mut self, x: usize, y: usize) -> ProcessorCycle {
        ProcessorCycle::skip_if(self.v_registers[x] != self.v_registers[y])
    }

    // LD I, nnn
    // Set I = nnn.
    fn execute_ld_i_nnn(&mut self, nnn: usize) -> ProcessorCycle {
        self.i_register = nnn;
        ProcessorCycle::Next
    }

    // JP V0, nnn
    // The program counter is set to nnn plus the value of V0.
    fn execute_jp_v0_nnn(&mut self, nnn: usize) -> ProcessorCycle {
        ProcessorCycle::Jump((self.v_registers[0] as usize) + nnn)
    }

    // RND Vx, nn
    // The interpreter generates a random number from 0 to 255,
    // which is then ANDed with the value nn. The results are stored in Vx.
    fn execute_rnd_vx_nn(&mut self, x: usize, nn: u8) -> ProcessorCycle {
        let mut rng = rand::thread_rng();
        self.v_registers[x] = rng.gen::<u8>() & nn;
        ProcessorCycle::Next
    }

    // DRW Vx, Vy, n
    // The interpreter reads n bytes from memory, starting at the address
    // stored in I. These bytes are then displayed as sprites on screen at
    // coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise
    // it is set to 0. If the sprite is positioned so part of it is outside
    // the coordinates of the display, it wraps around to the opposite side
    // of the screen.
    fn execute_drw_vx_vy_n(&mut self, x: usize, y: usize, n: usize) -> ProcessorCycle {
        // Get the (x, y) coords for our sprite
        let x_coord = self.v_registers[x] as usize;
        let y_coord = self.v_registers[y] as usize;
        // The last digit determines how many rows high our sprite is
        let num_rows = n;

        // Keep track if any pixels were flipped
        let mut flipped = false;
        // Iterate over each row of our sprite
        for y_line in 0..num_rows {
            // Determine which memory address our row's data is stored
            let addr = self.i_register + y_line;
            let row_pixels = self.memory.load(addr); // 8 pixels wide
                                                     // Iterate over each column in our row
                                                     // The rows in sprite are always 8 pixels wide, 1 byte
            for x_column in 0..8 {
                // Use a mask to fetch current pixel's bit. Only flip if a 1
                if (row_pixels & (0b1000_0000 >> x_column)) != 0 {
                    // Sprites should wrap around screen, so apply modulo
                    let x = (x_coord + x_column) % SCREEN_WIDTH;
                    let y = (y_coord + y_line) % SCREEN_HEIGHT;

                    // Get our pixel's index in the 1D screen array
                    let idx = x + SCREEN_WIDTH * y;
                    // Check if we're about to flip the pixel and set
                    flipped |= self.screen.get_pixel(idx);
                    // Invert the pixel with XOR
                    self.screen
                        .set_pixel(idx, self.screen.get_pixel(idx) ^ true);
                }
            }
        }
        self.v_registers[0x0f] = if flipped { 1 } else { 0 };
        ProcessorCycle::Next
    }

    // SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    fn execute_skp_vx(&mut self, x: usize) -> ProcessorCycle {
        ProcessorCycle::skip_if((self.keypad >> self.v_registers[x]) & 0x1 == 1)
    }

    // SKNP Vx
    // Skip next instruction if key with the value of Vx is NOT pressed.
    fn execute_sknp_vx(&mut self, x: usize) -> ProcessorCycle {
        ProcessorCycle::skip_if((self.keypad >> self.v_registers[x]) & 0x1 == 0)
    }

    // LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    fn execute_ld_vx_k(&mut self, x: usize) -> ProcessorCycle {
        self.keypad_wait = true;
        self.keypad_wait_index = x;
        ProcessorCycle::Next
    }

    // LD Vx, DT
    // Set Vx = delay timer value.
    fn execute_ld_vx_dt(&mut self, x: usize) -> ProcessorCycle {
        self.v_registers[x] = self.delay_timer;
        ProcessorCycle::Next
    }

    // LD DT, Vx
    // Set delay timer = Vx.
    fn execute_ld_dt_vx(&mut self, x: usize) -> ProcessorCycle {
        self.delay_timer = self.v_registers[x];
        ProcessorCycle::Next
    }

    // LD ST, Vx
    // Set sound timer = Vx.
    fn execute_ld_st_vx(&mut self, x: usize) -> ProcessorCycle {
        self.sound_timer = self.v_registers[x];
        ProcessorCycle::Next
    }

    // ADD I, Vx
    // Set I = I + Vx
    fn execute_add_i_vx(&mut self, x: usize) -> ProcessorCycle {
        self.i_register += self.v_registers[x] as usize;
        self.v_registers[0x0f] = if self.i_register > 0x0F00 { 1 } else { 0 };
        ProcessorCycle::Next
    }

    // LD F, Vx
    // Set I = location of Font sprite for digit Vx.
    fn execute_ld_f_vx(&mut self, x: usize) -> ProcessorCycle {
        self.i_register = (self.v_registers[x] as usize) * 5;
        ProcessorCycle::Next
    }

    // LD [I], Vx
    // The interpreter copies the values of registers V0 through Vx
    // into memory, starting at the address in register I.
    fn execute_ld_at_i_vx(&mut self, x: usize) -> ProcessorCycle {
        for i in 0..x + 1 {
            self.memory.store(self.i_register + i, self.v_registers[i]);
        }
        ProcessorCycle::Next
    }

    // LD Vx, [I]
    // The interpreter reads values from memory starting at location
    // I into registers V0 through Vx.
    fn execute_ld_vx_at_i(&mut self, x: usize) -> ProcessorCycle {
        for i in 0..x + 1 {
            self.v_registers[i] = self.memory.load(self.i_register + i);
        }
        ProcessorCycle::Next
    }

    // LD B, Vx
    // The interpreter takes the decimal value of Vx, and places
    // the hundreds digit in memory at location in I, the tens digit
    // at location I+1, and the ones digit at location I+2.
    fn execute_bcd_vx(&mut self, x: usize) -> ProcessorCycle {
        self.memory
            .store(self.i_register, self.v_registers[x] / 100);
        self.memory
            .store(self.i_register + 1, (self.v_registers[x] % 100) / 10);
        self.memory
            .store(self.i_register + 2, self.v_registers[x] % 10);
        ProcessorCycle::Next
    }

    fn execute_unknown(&self, opcode: u16) -> ProcessorCycle {
        ProcessorCycle::Error(
            DecodedOpcode::Unknown { opcode },
            "unknown opcode".to_string(),
        )
    }

    fn debug_rom(&mut self) {
        self.debugger.print_raw_rom();
        self.debugger.print_disassembled_rom();
    }

    fn debug_status(&mut self, address: usize, opcode: u16, decoded: &DecodedOpcode) {
        self.debugger.print_processor_status(
            self.screen.get_all_pixels(),
            &self.stack,
            self.stack_pointer,
            &self.v_registers,
            self.i_register,
            self.program_counter,
            self.delay_timer,
            self.sound_timer,
            self.keypad,
            address,
            opcode,
            &decoded,
        );
    }
}

enum ProcessorCycle {
    Error(DecodedOpcode, String),
    Next,
    Skip,
    Jump(usize),
}

impl ProcessorCycle {
    fn skip_if(condition: bool) -> ProcessorCycle {
        if condition {
            ProcessorCycle::Skip
        } else {
            ProcessorCycle::Next
        }
    }
}

#[cfg(test)]
#[path = "./processor_test.rs"]
mod processor_test;

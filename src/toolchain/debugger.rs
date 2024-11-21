use log::error;
use std::collections::{HashSet, VecDeque};

use super::decoder::DecodedOpcode;

use crate::constants::{
    RESERVED_MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, SEGMENTS_AFTER_PROGRAM_COUNTER, STACK_SIZE,
    V_REGISTERS_SIZE,
};

pub(crate) struct Debugger {
    rom: Vec<u8>,                     // The memory where the rom is loaded
    rom_size: usize,                  // Size of the rom loaded
    label_addresses: HashSet<usize>,  // List of labels for this rom
    opcode_addresses: HashSet<usize>, // List of opcodes for this rom
    current_address: usize,           // Current address in the rom
}

impl Debugger {
    pub(crate) fn new() -> Self {
        Self {
            rom: Vec::new(),
            rom_size: 0,
            label_addresses: HashSet::new(),
            opcode_addresses: HashSet::new(),
            current_address: 0,
        }
    }

    pub(crate) fn reset(&mut self, rom_bytes: &[u8], rom_size: usize) {
        self.disassemble(rom_bytes, rom_size);
    }

    fn fetch_opcode(&self, address: usize) -> u16 {
        let offset = address - RESERVED_MEMORY_SIZE;
        (self.rom[offset] as u16) << 8 | (self.rom[offset + 1] as u16)
    }

    fn disassemble(&mut self, rom_bytes: &[u8], rom_size: usize) {
        self.rom = rom_bytes.to_vec();
        self.rom_size = rom_size;

        let mut segments = VecDeque::new();
        segments.push_back(RESERVED_MEMORY_SIZE);

        while let Some(segment) = segments.pop_front() {
            self.current_address = segment;

            while self.current_address < self.rom_size + RESERVED_MEMORY_SIZE
                && !self.opcode_addresses.contains(&self.current_address)
            {
                let opcode = self.fetch_opcode(self.current_address);
                let decoded = DecodedOpcode::new(opcode);

                self.opcode_addresses.insert(self.current_address);
                self.current_address += 2;

                if let DecodedOpcode::Ret = decoded {
                    break;
                }

                match decoded {
                    DecodedOpcode::JpNnn { nnn } => {
                        self.current_address = nnn;
                        self.label_addresses.insert(nnn);
                    }
                    DecodedOpcode::CallNnn { nnn } => {
                        segments.push_back(self.current_address);
                        self.current_address = nnn;
                        self.label_addresses.insert(nnn);
                    }
                    DecodedOpcode::SeVxNn { .. }
                    | DecodedOpcode::SneVxNn { .. }
                    | DecodedOpcode::SeVxVy { .. }
                    | DecodedOpcode::SneVxVy { .. }
                    | DecodedOpcode::SkpVx { .. }
                    | DecodedOpcode::SknpVx { .. } => {
                        segments.push_back(self.current_address + 2);
                    }
                    DecodedOpcode::LdINnn { nnn } => {
                        self.label_addresses.insert(nnn);
                    }
                    DecodedOpcode::JpV0Nnn { .. } => {
                        error!("Encountered instruction 'JP V0, addr'. Unable to disassemble the code.");
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_registers_status(v_registers: &[u8; V_REGISTERS_SIZE]) -> String {
        let mut registers_status = String::new();
        for i in 0..V_REGISTERS_SIZE {
            let register_value = format!("{:02X}", v_registers[i]);
            registers_status.push_str(&format!("| V{:X}: {}        ", i, register_value));
            if i % 4 == 3 {
                registers_status.push_str(" |");
                if i < V_REGISTERS_SIZE - 1 {
                    registers_status.push_str("\n");
                }
            }
        }
        registers_status
    }

    fn get_keypad_status(keypad: u16) -> String {
        let mut keypad_status = String::new();
        for i in 0..16 {
            let key_value = format!("{:X}", (keypad >> i) & 0x1);
            keypad_status.push_str(&format!("| K{:X}: {}         ", i, key_value));
            if i % 4 == 3 {
                keypad_status.push_str(" |");
                if i < 15 {
                    keypad_status.push_str("\n");
                }
            }
        }
        keypad_status
    }

    fn get_stack_status(stack: &[usize; STACK_SIZE]) -> String {
        let mut stack_status = String::new();
        for i in 0..STACK_SIZE {
            let stack_value = format!("{:#06X}", stack[i]);
            stack_status.push_str(&format!("| SP{:02}: {}  ", i, stack_value));
            if i % 4 == 3 {
                stack_status.push_str(" |");
                if i < STACK_SIZE - 1 {
                    stack_status.push_str("\n");
                }
            }
        }
        stack_status
    }

    fn get_processor_status(
        stack_pointer: usize,
        i_register: usize,
        program_counter: usize,
        delay_timer: u8,
        sound_timer: u8,
    ) -> String {
        format!(
            "| PC: {:#06X}    | I: {:#06X}     | SP: {:02}   | DT: {:02}   | ST: {:02}   |",
            program_counter, i_register, stack_pointer, delay_timer, sound_timer
        )
    }

    fn get_screen_status(screen_pixels: &[bool]) -> String {
        let mut screen_status = String::new();
        for y in 0..SCREEN_HEIGHT {
            screen_status.push_str("|");
            for x in 0..SCREEN_WIDTH {
                let idx = x + SCREEN_WIDTH * y;
                screen_status.push_str(if screen_pixels[idx] { "X" } else { " " });
            }
            screen_status.push_str("|");
            if y < SCREEN_HEIGHT - 1 {
                screen_status.push_str("\n");
            }
        }
        screen_status
    }

    fn apply_highlight(
        status: &mut String,
        address: usize,
        highlight: Option<usize>,
        highlight_symbol: Option<String>,
    ) {
        if let Some(highlight_address) = highlight {
            if address == highlight_address {
                if let Some(highlight_symbol) = highlight_symbol {
                    if let Some(pos) = status.rfind('|') {
                        status.replace_range(
                            (pos - highlight_symbol.len() - 1)..=pos,
                            &format!("{} |", highlight_symbol),
                        );
                    }
                }
            }
        }
    }

    fn get_opcode_status(
        address: usize,
        opcode: u16,
        decoded: &DecodedOpcode,
        highlight: Option<usize>,
        highlight_symbol: Option<String>,
    ) -> String {
        let address_string = format!("{:#06X}", address);
        let opcode_string = format!("{:04X}", opcode);
        let decoded_string = format!("{:20}", decoded.to_string());

        let mut status = format!(
            "| AD: {}    | OPCODE: {}  | DECODED: {}  |",
            address_string, opcode_string, decoded_string
        );

        Self::apply_highlight(&mut status, address, highlight, highlight_symbol);
        status
    }

    fn get_byte_status(
        address: usize,
        byte: u8,
        highlight: Option<usize>,
        highlight_symbol: Option<String>,
    ) -> String {
        let mut status = format!(
            "| AD: {:#06X}    | BYTE: {:#04X}    | BINARY: {:#010b}             |",
            address, byte, byte
        );

        Self::apply_highlight(&mut status, address, highlight, highlight_symbol);
        status
    }

    fn get_raw_rom(&mut self) -> String {
        let mut status = String::new();
        for (i, byte) in self.rom.iter().enumerate() {
            if i < self.rom_size {
                status.push_str(&Self::get_byte_status(
                    i + RESERVED_MEMORY_SIZE,
                    *byte,
                    None,
                    None,
                ));
                if i < self.rom_size - 1 {
                    status.push_str("\n");
                }
            } else {
                break;
            }
        }
        status
    }

    fn get_disassembled_rom(&mut self) -> String {
        self.current_address = RESERVED_MEMORY_SIZE;
        let mut output = String::new();

        while self.current_address < self.rom_size + RESERVED_MEMORY_SIZE {
            // Output the byte status
            if !self.opcode_addresses.contains(&self.current_address) {
                let address = self.current_address;
                let byte = self.rom[self.current_address - RESERVED_MEMORY_SIZE];
                let (highlight_address, highlight_symbol) =
                    if self.label_addresses.contains(&address) {
                        // Output the label status
                        (Some(address), Some(format!("* LB-{:04X}", address)))
                    } else {
                        (None, None)
                    };
                output.push_str(&format!(
                    "{}\n",
                    Self::get_byte_status(address, byte, highlight_address, highlight_symbol)
                ));
                self.current_address += 1;
                continue;
            }

            // Output the opcode status
            let address = self.current_address;
            let opcode = self.fetch_opcode(self.current_address);
            let decoded = &DecodedOpcode::new(opcode);
            output.push_str(&format!(
                "{}\n",
                Self::get_opcode_status(address, opcode, &decoded, None, None)
            ));
            self.current_address += 2;
        }

        if output.ends_with('\n') {
            output.truncate(output.len() - 1);
        }
        output
    }

    fn get_disassembled_rom_after_program_counter(
        &mut self,
        program_counter: usize,
        total: usize,
    ) -> String {
        let mut output = String::new();
        let mut address = program_counter;
        let mut count = 0;

        while address < self.rom_size + RESERVED_MEMORY_SIZE && count < total {
            count += 1;

            // Output the byte status
            if !self.opcode_addresses.contains(&address) {
                let byte = self.rom[address - RESERVED_MEMORY_SIZE];
                output.push_str(&format!(
                    "{}\n",
                    Self::get_byte_status(address, byte, None, None)
                ));
                address += 1;
                continue;
            }

            // Output the opcode status
            let opcode = self.fetch_opcode(address);
            let decoded = &DecodedOpcode::new(opcode);
            output.push_str(&format!(
                "{}\n",
                Self::get_opcode_status(
                    address,
                    opcode,
                    &decoded,
                    Some(program_counter),
                    Some("* PC".to_string())
                )
            ));
            address += 2;
        }

        if output.ends_with('\n') {
            output.truncate(output.len() - 1);
        }
        output
    }

    pub(crate) fn print_raw_rom(&mut self) {
        log::debug!(
            r#"
- raw rom --------------------------------------------------------
{}
------------------------------------------------------------------"#,
            self.get_raw_rom()
        );
    }

    pub(crate) fn print_disassembled_rom(&mut self) {
        log::debug!(
            r#"
- disassembled rom -----------------------------------------------
{}
------------------------------------------------------------------"#,
            self.get_disassembled_rom(),
        );
    }

    pub(crate) fn print_processor_status(
        &mut self,
        screen_pixels: &[bool],
        stack: &[usize; STACK_SIZE],
        stack_pointer: usize,
        v_registers: &[u8; V_REGISTERS_SIZE],
        i_register: usize,
        program_counter: usize,
        delay_timer: u8,
        sound_timer: u8,
        keypad: u16,
        address: usize,
        opcode: u16,
        decoded: &DecodedOpcode,
    ) {
        log::debug!(
            r#"
- screen status --------------------------------------------------
{}
------------------------------------------------------------------"#,
            Self::get_screen_status(screen_pixels)
        );

        log::debug!(
            r#"
- disassembled rom after pc --------------------------------------
{}
------------------------------------------------------------------"#,
            self.get_disassembled_rom_after_program_counter(
                program_counter,
                SEGMENTS_AFTER_PROGRAM_COUNTER
            )
        );

        log::debug!(
            r#"
- registers status -----------------------------------------------
{}
------------------------------------------------------------------"#,
            Self::get_registers_status(v_registers),
        );

        log::debug!(
            r#"
- keypad status --------------------------------------------------
{}
------------------------------------------------------------------"#,
            Self::get_keypad_status(keypad),
        );

        log::debug!(
            r#"
- stack status ---------------------------------------------------
{}
------------------------------------------------------------------"#,
            Self::get_stack_status(stack),
        );

        log::debug!(
            r#"
- processor status -----------------------------------------------
{}
------------------------------------------------------------------"#,
            Self::get_processor_status(
                stack_pointer,
                i_register,
                program_counter,
                delay_timer,
                sound_timer
            ),
        );

        log::debug!(
            r#"
- processor execute ----------------------------------------------
{}
------------------------------------------------------------------"#,
            Self::get_opcode_status(
                address,
                opcode,
                decoded,
                Some(program_counter),
                Some("* PC".to_string())
            ),
        );
    }
}

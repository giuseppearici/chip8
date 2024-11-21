pub enum DecodedOpcode {
    // 00E0 | CLS | Clear video memory
    Cls,

    // 00EE | RET | Return from subroutine
    Ret,

    // 0NNN | SYS NNN | Call machine subroutine at NNN (ignored in modern interpreters)
    SysNnn { nnn: usize },

    // 2NNN | CALL NNN | Call CHIP-8 subroutine at NNN
    CallNnn { nnn: usize },

    // 1NNN | JP NNN | Jump to address NNN
    JpNnn { nnn: usize },

    // BNNN | JP V0, NNN | Jump to address NNN + V0
    JpV0Nnn { nnn: usize },

    // 3XNN | SE VX, NN | Skip next instruction if VX == NN
    SeVxNn { vx: usize, nn: u8 },

    // 4XNN | SNE VX, NN | Skip next instruction if VX != NN
    SneVxNn { vx: usize, nn: u8 },

    // 5XY0 | SE VX, VY | Skip next instruction if VX == VY
    SeVxVy { vx: usize, vy: usize },

    // 9XY0 | SNE VX, VY | Skip next instruction if VX != VY
    SneVxVy { vx: usize, vy: usize },

    // EX9E | SKP VX | Skip next instruction if key(VX) is pressed
    SkpVx { vx: usize },

    // EXA1 | SKNP VX | Skip next instruction if key(VX) is not pressed
    SknpVx { vx: usize },

    // FX0A | LD VX, K | Wait for key press, store key pressed in VX
    LdVxK { vx: usize },

    // 6XNN | LD VX, NN | VX = NN
    LdVxNn { vx: usize, nn: u8 },

    // 8XY0 | LD VX, VY | VX = VY
    LdVxVy { vx: usize, vy: usize },

    // FX07 | LD VX, DT | VX = DT
    LdVxDt { vx: usize },

    // FX15 | LD DT, VX | DT = VX
    LdDtVx { vx: usize },

    // FX18 | LD ST, VX | ST = VX
    LdStVx { vx: usize },

    // ANNN | LD I, NNN | I = NNN
    LdINnn { nnn: usize },

    // FX29 | LD F, VX | I = address of 4x5 font character in VX (0...F)
    LdFVx { vx: usize },

    // FX55 | LD [I], VX | Store V0...VX (inclusive) to memory starting at I; 'I' remains unchanged
    LdAtIVx { vx: usize },

    // FX65 | LD VX, [I] | Load V0...VX (inclusive) from memory starting at I; 'I' remains unchanged
    LdVxAtI { vx: usize },

    // FX1E | ADD I, VX | I = I + VX; VF = 1 if I > 0xFFF else 0
    AddIVx { vx: usize },

    // 7XNN | ADD VX, NN | VX = VX + NN
    AddVxNn { vx: usize, nn: u8 },

    // 8XY4 | ADD VX, VY | VX = VX + VY; VF = 1 if overflow else 0
    AddVxVy { vx: usize, vy: usize },

    // 8XY5 | SUB VX, VY | VX = VX - VY; VF = 1 if not borrow else 0
    SubVxVy { vx: usize, vy: usize },

    // 8XY7 | SUBN VX, VY | VX = VY - VX; VF = 1 if not borrow else 0
    SubnVxVy { vx: usize, vy: usize },

    // 8XY1 | OR VX, VY | VX = VX OR VY
    OrVxVy { vx: usize, vy: usize },

    // 8XY2 | AND VX, VY | VX = VX AND VY
    AndVxVy { vx: usize, vy: usize },

    // 8XY3 | XOR VX, VY | VX = VX XOR VY
    XorVxVy { vx: usize, vy: usize },

    // 8XY6 | SHR VX | VF = LSB(VX); VX = VX >> 1
    ShrVx { vx: usize },

    // 8XYE | SHL VX | VF = MSB(VX); VX = VX << 1
    ShlVx { vx: usize },

    // FX33 | BCD VX | Store BCD repr of VX at I (100), I+1 (10), and I+2 (1); 'I' remains unchanged
    BcdVx { vx: usize },

    // CXNN | RND VX, NN | VX = RND() AND NN
    RndVxNn { vx: usize, nn: u8 },

    // DXYN | DRW VX, VY, N | Draw 8xN sprite at I to VX, VY; VF = 1 if collision else 0
    DrwVxVyN { vx: usize, vy: usize, n: usize },

    // ____ | UNKNOWN | Unknown opcode
    Unknown { opcode: u16 },
}

impl DecodedOpcode {
    pub(crate) fn new(opcode: u16) -> Self {
        let nibbles = (
            (opcode & 0xF000) >> 12u8,
            (opcode & 0x0F00) >> 8u8,
            (opcode & 0x00F0) >> 4u8,
            opcode & 0x000F,
        );

        let vx = nibbles.1 as usize;
        let vy = nibbles.2 as usize;
        let n = nibbles.3 as usize;
        let nn = (nibbles.2 << 4u8 | nibbles.3) as u8;
        let nnn = (nibbles.1 << 8u8 | nibbles.2 << 4u8 | nibbles.3) as usize;

        match nibbles {
            // OPCODE: 00E0  => DECODED: CLS
            (0x0, 0x0, 0xE, 0x0) => Self::Cls,

            // OPCODE: 00EE  => DECODED: RET
            (0x0, 0x0, 0xE, 0xE) => Self::Ret,

            // OPCODE: 0NNN  => DECODED: SYS NNN
            (0x0, _, _, _) => Self::SysNnn { nnn },

            // OPCODE: 1NNN  => DECODED: JP NNN
            (0x1, _, _, _) => Self::JpNnn { nnn },

            // OPCODE: 2NNN  => DECODED: CALL NNN
            (0x2, _, _, _) => Self::CallNnn { nnn },

            // OPCODE: BNNN  => DECODED: JP V0, NNN
            (0xB, _, _, _) => Self::JpV0Nnn { nnn },

            // OPCODE: 3XNN  => DECODED: SE VX, NN
            (0x3, _, _, _) => Self::SeVxNn { vx, nn },

            // OPCODE: 4XNN  => DECODED: SNE VX, NN
            (0x4, _, _, _) => Self::SneVxNn { vx, nn },

            // OPCODE: 5XY0  => DECODED: SE VX, VY
            (0x5, _, _, 0x0) => Self::SeVxVy { vx, vy },

            // OPCODE: 9XY0  => DECODED: SNE VX, VY
            (0x9, _, _, 0x0) => Self::SneVxVy { vx, vy },

            // OPCODE: EX9E  => DECODED: SKP VX
            (0xE, _, 0x9, 0xE) => Self::SkpVx { vx },

            // OPCODE: EXA1  => DECODED: SKNP VX
            (0xE, _, 0xA, 0x1) => Self::SknpVx { vx },

            // OPCODE: FX0A  => DECODED: LD VX, K
            (0xF, _, 0x0, 0xA) => Self::LdVxK { vx },

            // OPCODE: 6XNN  => DECODED: LD VX, NN
            (0x6, _, _, _) => Self::LdVxNn { vx, nn },

            // OPCODE: 8XY0  => DECODED: LD VX, VY
            (0x8, _, _, 0x0) => Self::LdVxVy { vx, vy },

            // OPCODE: FX07  => DECODED: LD VX, DT
            (0xF, _, 0x0, 0x7) => Self::LdVxDt { vx },

            // OPCODE: FX15  => DECODED: LD DT, VX
            (0xF, _, 0x1, 0x5) => Self::LdDtVx { vx },

            // OPCODE: FX18  => DECODED: LD ST, VX
            (0xF, _, 0x1, 0x8) => Self::LdStVx { vx },

            // OPCODE: ANNN  => DECODED: LD I, NNN
            (0xA, _, _, _) => Self::LdINnn { nnn },

            // OPCODE: FX29  => DECODED: LD F, VX
            (0xF, _, 0x2, 0x9) => Self::LdFVx { vx },

            // OPCODE: FX55  => DECODED: LD [I], VX
            (0xF, _, 0x5, 0x5) => Self::LdAtIVx { vx },

            // OPCODE: FX65  => DECODED: LD VX, [I]
            (0xF, _, 0x6, 0x5) => Self::LdVxAtI { vx },

            // OPCODE: FX1E  => DECODED: ADD I, VX
            (0xF, _, 0x1, 0xE) => Self::AddIVx { vx },

            // OPCODE: 7XNN  => DECODED: ADD VX, NN
            (0x7, _, _, _) => Self::AddVxNn { vx, nn },

            // OPCODE: 8XY4  => DECODED: ADD VX, VY
            (0x8, _, _, 0x4) => Self::AddVxVy { vx, vy },

            // OPCODE: 8XY5  => DECODED: SUB VX, VY
            (0x8, _, _, 0x5) => Self::SubVxVy { vx, vy },

            // OPCODE: 8XY7  => DECODED: SUBN VX, VY
            (0x8, _, _, 0x7) => Self::SubnVxVy { vx, vy },

            // OPCODE: 8XY1  => DECODED: OR VX, VY
            (0x8, _, _, 0x1) => Self::OrVxVy { vx, vy },

            // OPCODE: 8XY2  => DECODED: AND VX, VY
            (0x8, _, _, 0x2) => Self::AndVxVy { vx, vy },

            // OPCODE: 8XY3  => DECODED: XOR VX, VY
            (0x8, _, _, 0x3) => Self::XorVxVy { vx, vy },

            // OPCODE: 8XY6  => DECODED: SHR VX
            (0x8, _, _, 0x6) => Self::ShrVx { vx },

            // OPCODE: 8XYE  => DECODED: SHL VX
            (0x8, _, _, 0xE) => Self::ShlVx { vx },

            // OPCODE: FX33  => DECODED: BCD VX
            (0xF, _, 0x3, 0x3) => Self::BcdVx { vx },

            // OPCODE: CXNN  => DECODED: RND VX, NN
            (0xC, _, _, _) => Self::RndVxNn { vx, nn },

            // OPCODE: DXYN  => DECODED: DRW VX, VY, N
            (0xD, _, _, _) => Self::DrwVxVyN { vx, vy, n },

            // OPCODE: ____  => DECODED: UNKNOWN
            _ => Self::Unknown { opcode },
        }
    }

    pub(crate) fn to_string(&self) -> String {
        match self {
            Self::Cls => "CLS".to_string(),
            Self::Ret => "RET".to_string(),
            Self::SysNnn { nnn } => format!("SYS {:#06X}", nnn),
            Self::CallNnn { nnn } => format!("CALL {:#06X}", nnn),
            Self::JpNnn { nnn } => format!("JP {:#06X}", nnn),
            Self::JpV0Nnn { nnn } => format!("JP V0, {:#06X}", nnn),
            Self::SeVxNn { vx, nn } => format!("SE V{:X}, {:#04X}", vx, nn),
            Self::SneVxNn { vx, nn } => format!("SNE V{:X}, {:#04X}", vx, nn),
            Self::SeVxVy { vx, vy } => format!("SE V{:X}, V{:X}", vx, vy),
            Self::SneVxVy { vx, vy } => format!("SNE V{:X}, V{:X}", vx, vy),
            Self::SkpVx { vx } => format!("SKP V{:X}", vx),
            Self::SknpVx { vx } => format!("SKNP V{:X}", vx),
            Self::LdVxK { vx } => format!("LD V{:X}, K", vx),
            Self::LdVxNn { vx, nn } => format!("LD V{:X}, {:#04X}", vx, nn),
            Self::LdVxVy { vx, vy } => format!("LD V{:X}, V{:X}", vx, vy),
            Self::LdVxDt { vx } => format!("LD V{:X}, DT", vx),
            Self::LdDtVx { vx } => format!("LD DT, V{:X}", vx),
            Self::LdStVx { vx } => format!("LD ST, V{:X}", vx),
            Self::LdINnn { nnn } => format!("LD I, {:#06X}", nnn),
            Self::LdFVx { vx } => format!("LD F, V{:X}", vx),
            Self::LdAtIVx { vx } => format!("LD [I], V{:X}", vx),
            Self::LdVxAtI { vx } => format!("LD V{:X}, [I]", vx),
            Self::AddIVx { vx } => format!("ADD I, V{:X}", vx),
            Self::AddVxNn { vx, nn } => format!("ADD V{:X}, {:#04X}", vx, nn),
            Self::AddVxVy { vx, vy } => format!("ADD V{:X}, V{:X}", vx, vy),
            Self::SubVxVy { vx, vy } => format!("SUB V{:X}, V{:X}", vx, vy),
            Self::SubnVxVy { vx, vy } => format!("SUBN V{:X}, V{:X}", vx, vy),
            Self::OrVxVy { vx, vy } => format!("OR V{:X}, V{:X}", vx, vy),
            Self::AndVxVy { vx, vy } => format!("AND V{:X}, V{:X}", vx, vy),
            Self::XorVxVy { vx, vy } => format!("XOR V{:X}, V{:X}", vx, vy),
            Self::ShrVx { vx } => format!("SHR V{:X}", vx),
            Self::ShlVx { vx } => format!("SHL V{:X}", vx),
            Self::BcdVx { vx } => format!("BCD V{:X}", vx),
            Self::RndVxNn { vx, nn } => format!("RND V{:X}, {:#04X}", vx, nn),
            Self::DrwVxVyN { vx, vy, n } => format!("DRW V{:X}, V{:X}, {:0}", vx, vy, n),
            Self::Unknown { opcode } => format!("UNKNOWN {:04X}", opcode),
        }
    }
}

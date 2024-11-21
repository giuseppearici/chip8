use super::super::memory::FONT_SPRITES;
use super::*;

const PROGRAM_COUNTER_START: usize = 0xF00;
const PROGRAM_COUNTER_NEXT: usize = PROGRAM_COUNTER_START + OPCODE_SIZE;
const PROGRAM_COUNTER_SKIP: usize = PROGRAM_COUNTER_START + (2 * OPCODE_SIZE);

fn build_processor() -> Processor {
    let mut processor = Processor::new();
    processor.program_counter = PROGRAM_COUNTER_START;
    processor.v_registers = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7];
    processor
}

#[test]
fn test_initial_state() {
    let processor = Processor::new();
    assert_eq!(processor.program_counter, 0x200);
    assert_eq!(processor.stack_pointer, 0);
    assert_eq!(processor.stack, [0; 16]);
    // First char in font: 0
    assert_eq!(processor.memory.load(0), 0xF0);
    assert_eq!(processor.memory.load(1), 0x90);
    assert_eq!(processor.memory.load(2), 0x90);
    assert_eq!(processor.memory.load(3), 0x90);
    assert_eq!(processor.memory.load(4), 0xF0);
    // Last char in font: F
    assert_eq!(processor.memory.load(FONT_SPRITES.len() - 5), 0xF0);
    assert_eq!(processor.memory.load(FONT_SPRITES.len() - 4), 0x80);
    assert_eq!(processor.memory.load(FONT_SPRITES.len() - 3), 0xF0);
    assert_eq!(processor.memory.load(FONT_SPRITES.len() - 2), 0x80);
    assert_eq!(processor.memory.load(FONT_SPRITES.len() - 1), 0x80);
}

#[test]
fn test_load_data() {
    let mut processor = Processor::new();
    processor.memory.reset(&[1, 2, 3], 3);
    assert_eq!(processor.memory.load(0x200), 1);
    assert_eq!(processor.memory.load(0x201), 2);
    assert_eq!(processor.memory.load(0x202), 3);
}

// CLS
#[test]
fn test_execute_opcode_00e0() {
    let mut processor = build_processor();
    for index in 0..SCREEN_WIDTH * SCREEN_HEIGHT {
        processor.screen.set_pixel(index, true);
    }
    processor.execute_opcode(processor.decode_opcode(0x00e0));

    for index in 0..SCREEN_WIDTH * SCREEN_HEIGHT {
        assert_eq!(processor.screen.get_pixel(index), false);
    }

    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// RET
#[test]
fn test_execute_opcode_00ee() {
    let mut processor = Processor::new();
    processor.stack_pointer = 5;
    processor.stack[4] = 0x6666;
    processor.execute_opcode(processor.decode_opcode(0x00ee));
    assert_eq!(processor.stack_pointer, 4);
    assert_eq!(processor.program_counter, 0x6666);
}

// JP
#[test]
fn test_execute_opcode_1nnn() {
    let mut processor = Processor::new();
    processor.execute_opcode(processor.decode_opcode(0x1666));
    assert_eq!(processor.program_counter, 0x0666);
}

// CALL
#[test]
fn test_execute_opcode_2nnn() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x2666));
    assert_eq!(processor.program_counter, 0x0666);
    assert_eq!(processor.stack_pointer, 1);
    assert_eq!(processor.stack[0], PROGRAM_COUNTER_NEXT);
}

// SE VX, byte
#[test]
fn test_execute_opcode_3xkk() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x3201));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_SKIP);
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x3200));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// SNE VX, byte
#[test]
fn test_execute_opcode_4xkk() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x4200));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_SKIP);
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x4201));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// SE VX, VY
#[test]
fn test_execute_opcode_5xy0() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x5540));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_SKIP);
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x5500));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD Vx, byte
#[test]
fn test_execute_opcode_6xkk() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x65ff));
    assert_eq!(processor.v_registers[5], 0xff);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// ADD Vx, byte
#[test]
fn test_execute_opcode_7xkk() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x75f0));
    assert_eq!(processor.v_registers[5], 0xf2);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD Vx, Vy
#[test]
fn test_execute_opcode_8xy0() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x8050));
    assert_eq!(processor.v_registers[0], 0x02);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

fn check_math(v1: u8, v2: u8, op: u16, result: u8, vf: u8) {
    let mut processor = build_processor();
    processor.v_registers[0] = v1;
    processor.v_registers[1] = v2;
    processor.v_registers[0x0f] = 0;
    processor.execute_opcode(processor.decode_opcode(0x8010 + op));
    assert_eq!(processor.v_registers[0], result);
    assert_eq!(processor.v_registers[0x0f], vf);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// OR Vx, Vy
#[test]
fn test_execute_opcode_8xy1() {
    // 0x0F or 0xF0 == 0xFF
    check_math(0x0F, 0xF0, 1, 0xFF, 0);
}

// AND Vx, Vy
#[test]
fn test_execute_opcode_8xy2() {
    // 0x0F and 0xFF == 0x0F
    check_math(0x0F, 0xFF, 2, 0x0F, 0);
}

// XOR Vx, Vy
#[test]
fn test_execute_opcode_8xy3() {
    // 0x0F xor 0xFF == 0xF0
    check_math(0x0F, 0xFF, 3, 0xF0, 0);
}

// ADD Vx, Vy
#[test]
fn test_execute_opcode_8xy4() {
    check_math(0x0F, 0x0F, 4, 0x1E, 0);
    check_math(0xFF, 0xFF, 4, 0xFE, 1);
}

// SUB Vx, Vy
#[test]
fn test_execute_opcode_8xy5() {
    check_math(0x0F, 0x01, 5, 0x0E, 1);
    check_math(0x0F, 0xFF, 5, 0x10, 0);
}

// SHR Vx
#[test]
fn test_execute_opcode_8x06() {
    // 4 >> 1 == 2
    check_math(0x04, 0, 6, 0x02, 0);
    // 5 >> 1 == 2 with carry
    check_math(0x05, 0, 6, 0x02, 1);
}

// SUBN Vx, Vy
#[test]
fn test_execute_opcode_8xy7() {
    check_math(0x01, 0x0F, 7, 0x0E, 1);
    check_math(0xFF, 0x0F, 7, 0x10, 0);
}

// SHL Vx
#[test]
fn test_execute_opcode_8x0e() {
    check_math(0b11000000, 0, 0x0e, 0b10000000, 1);
    check_math(0b00000111, 0, 0x0e, 0b00001110, 0);
}

// SNE VX, VY
#[test]
fn test_execute_opcode_9xy0() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x90e0));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_SKIP);
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0x9010));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD I, byte
#[test]
fn test_execute_opcode_annn() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0xa123));
    assert_eq!(processor.i_register, 0x123);
}

// JP V0, addr
#[test]
fn test_execute_opcode_bnnn() {
    let mut processor = build_processor();
    processor.v_registers[0] = 3;
    processor.execute_opcode(processor.decode_opcode(0xb123));
    assert_eq!(processor.program_counter, 0x126);
}

// RND Vx, byte
// Generates random u8, then ANDs it with kk.
// We can't test randomness, but we can test the AND.
#[test]
fn test_execute_opcode_cxkk() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0xc000));
    assert_eq!(processor.v_registers[0], 0);
    processor.execute_opcode(processor.decode_opcode(0xc00f));
    assert_eq!(processor.v_registers[0] & 0xf0, 0);
}

// DRW Vx, Vy, nibble
#[test]
fn test_execute_opcode_dxyn() {
    let mut processor = build_processor();
    processor.i_register = 0;
    processor.memory.store(0, 0b11111111);
    processor.memory.store(1, 0b00000000);
    processor.screen.set_pixel(0, true);
    processor.screen.set_pixel(SCREEN_WIDTH, true);
    processor.v_registers[0] = 0;
    processor.execute_opcode(processor.decode_opcode(0xd002));
    assert_eq!(processor.screen.get_pixel(0), false);
    assert_eq!(processor.screen.get_pixel(1), true);
    assert_eq!(processor.screen.get_pixel(SCREEN_WIDTH), true);
    assert_eq!(processor.screen.get_pixel(SCREEN_WIDTH + 1), false);
    assert_eq!(processor.v_registers[0x0f], 1);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

#[test]
fn test_execute_opcode_dxyn_wrap_horizontal() {
    let mut processor = build_processor();
    let x = SCREEN_WIDTH - 4;
    processor.i_register = 0;
    processor.memory.store(0, 0b11111111);
    processor.v_registers[0] = x as u8;
    processor.v_registers[1] = 0;
    processor.execute_opcode(processor.decode_opcode(0xd011));
    assert_eq!(processor.screen.get_pixel(x - 1), false);
    assert_eq!(processor.screen.get_pixel(x), true);
    assert_eq!(processor.screen.get_pixel(x + 1), true);
    assert_eq!(processor.screen.get_pixel(x + 2), true);
    assert_eq!(processor.screen.get_pixel(x + 3), true);
    assert_eq!(processor.screen.get_pixel(0), true);
    assert_eq!(processor.screen.get_pixel(1), true);
    assert_eq!(processor.screen.get_pixel(2), true);
    assert_eq!(processor.screen.get_pixel(3), true);
    assert_eq!(processor.screen.get_pixel(4), false);
    assert_eq!(processor.v_registers[0x0f], 0);
}

// DRW Vx, Vy, nibble
#[test]
fn test_execute_opcode_dxyn_wrap_vertical() {
    let mut processor = build_processor();
    let y = SCREEN_HEIGHT - 1;
    processor.i_register = 0;
    processor.memory.store(0, 0b11111111);
    processor.memory.store(1, 0b11111111);
    processor.v_registers[0] = 0;
    processor.v_registers[1] = y as u8;
    processor.execute_opcode(processor.decode_opcode(0xd012));
    assert_eq!(processor.screen.get_pixel(y * SCREEN_WIDTH), true);
    assert_eq!(processor.screen.get_pixel(0), true);
    assert_eq!(processor.v_registers[0x0f], 0);
}

// SKP Vx
#[test]
fn test_execute_opcode_ex9e() {
    let mut processor = build_processor();
    processor.keypad = 0b0000001000000000;
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xe59e));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_SKIP);

    let mut processor = build_processor();
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xe59e));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// SKNP Vx
#[test]
fn test_execute_opcode_exa1() {
    let mut processor = build_processor();
    processor.keypad = 0b0000001000000000;
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xe5a1));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);

    let mut processor = build_processor();
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xe5a1));
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_SKIP);
}

// LD Vx, DT
#[test]
fn test_execute_opcode_fx07() {
    let mut processor = build_processor();
    processor.delay_timer = 20;
    processor.execute_opcode(processor.decode_opcode(0xf507));
    assert_eq!(processor.v_registers[5], 20);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD Vx, K
#[test]
fn test_execute_opcode_fx0a() {
    let mut processor = build_processor();
    processor.execute_opcode(processor.decode_opcode(0xf50a));
    assert_eq!(processor.keypad_wait, true);
    assert_eq!(processor.keypad_wait_index, 5);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);

    // Tick with no keypress doesn't do anything
    processor.tick(0x0);
    assert_eq!(processor.keypad_wait, true);
    assert_eq!(processor.keypad_wait_index, 5);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);

    // Tick with a keypress finishes wait and loads
    // first pressed key into vx
    processor.tick(0xffff);
    assert_eq!(processor.keypad_wait, false);
    assert_eq!(processor.v_registers[5], 0);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD DT, vX
#[test]
fn test_execute_opcode_fx15() {
    let mut processor = build_processor();
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xf515));
    assert_eq!(processor.delay_timer, 9);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD ST, vX
#[test]
fn test_execute_opcode_fx18() {
    let mut processor = build_processor();
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xf518));
    assert_eq!(processor.sound_timer, 9);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// ADD I, Vx
#[test]
fn test_execute_opcode_fx1e() {
    let mut processor = build_processor();
    processor.v_registers[5] = 9;
    processor.i_register = 9;
    processor.execute_opcode(processor.decode_opcode(0xf51e));
    assert_eq!(processor.i_register, 18);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD F, Vx
#[test]
fn test_execute_opcode_fx29() {
    let mut processor = build_processor();
    processor.v_registers[5] = 9;
    processor.execute_opcode(processor.decode_opcode(0xf529));
    assert_eq!(processor.i_register, 5 * 9);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD B, Vx
#[test]
fn test_execute_opcode_fx33() {
    let mut processor = build_processor();
    processor.v_registers[5] = 123;
    processor.i_register = 1000;
    processor.execute_opcode(processor.decode_opcode(0xf533));
    assert_eq!(processor.memory.load(1000), 1);
    assert_eq!(processor.memory.load(1001), 2);
    assert_eq!(processor.memory.load(1002), 3);
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD [I], Vx
#[test]
fn test_execute_opcode_fx55() {
    let mut processor = build_processor();
    processor.i_register = 1000;
    processor.execute_opcode(processor.decode_opcode(0xff55));
    for i in 0..16 {
        assert_eq!(processor.memory.load(1000 + i), processor.v_registers[i]);
    }
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

// LD Vx, [I]
#[test]
fn test_execute_opcode_fx65() {
    let mut processor = build_processor();
    for i in 0..16usize {
        processor.memory.store(1000 + i, i as u8);
    }
    processor.i_register = 1000;
    processor.execute_opcode(processor.decode_opcode(0xff65));

    for i in 0..16usize {
        assert_eq!(processor.v_registers[i], processor.memory.load(1000 + i));
    }
    assert_eq!(processor.program_counter, PROGRAM_COUNTER_NEXT);
}

#[test]
fn test_timers() {
    let mut processor = build_processor();
    processor.delay_timer = 200;
    processor.sound_timer = 100;
    processor.tick(0x0000);
    processor.delay_timer -= 1;
    processor.sound_timer -= 1;
    assert_eq!(processor.delay_timer, 199);
    assert_eq!(processor.sound_timer, 99);
}

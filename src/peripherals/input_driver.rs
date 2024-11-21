use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

//  Keyboard              Chip8
//  +---+---+---+---+     +---+---+---+---+
//  | 1 | 2 | 3 | 4 |     | 1 | 2 | 3 | C |
//  +---+---+---+---+     +---+---+---+---+
//  | Q | W | E | R |     | 4 | 5 | 6 | D |
//  +---+---+---+---+  >  +---+---+---+---+
//  | A | S | D | F |     | 7 | 8 | 9 | E |
//  +---+---+---+---+     +---+---+---+---+
//  | Z | X | C | V |     | A | 0 | B | F |
//  +---+---+---+---+     +---+---+---+---+

pub(crate) struct InputDriver {
    events: sdl2::EventPump,
}

impl InputDriver {
    pub(crate) fn new(sdl_context: &sdl2::Sdl) -> Self {
        InputDriver {
            events: sdl_context.event_pump().unwrap(),
        }
    }

    pub(crate) fn poll(&mut self) -> Result<u16, ()> {
        for event in self.events.poll_iter() {
            if let Event::Quit { .. } = event {
                return Err(());
            };
        }

        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        // Bit: 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
        // Key:  F  E  D  C  B  A 9 8 7 6 5 4 3 2 1 0
        let mut chip8_keys: u16 = 0;
        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0b0000_0000_0000_0010), // Bit  1 active => Key 1 pressed
                Keycode::Num2 => Some(0b0000_0000_0000_0100), // Bit  2 active => Key 2 pressed
                Keycode::Num3 => Some(0b0000_0000_0000_1000), // Bit  3 active => Key 3 pressed
                Keycode::Num4 => Some(0b0001_0000_0000_0000), // Bit 12 active => Key C pressed

                Keycode::Q => Some(0b0000_0000_0001_0000), // Bit  4 active => Key 4 pressed
                Keycode::W => Some(0b0000_0000_0010_0000), // Bit  5 active => Key 5 pressed
                Keycode::E => Some(0b0000_0000_0100_0000), // Bit  6 active => Key 6 pressed
                Keycode::R => Some(0b0010_0000_0000_0000), // Bit 13 active => Key D pressed

                Keycode::A => Some(0b0000_0000_1000_0000), // Bit  7 active => Key 7 pressed
                Keycode::S => Some(0b0000_0001_0000_0000), // Bit  8 active => Key 8 pressed
                Keycode::D => Some(0b0000_0010_0000_0000), // Bit  9 active => Key 9 pressed
                Keycode::F => Some(0b0100_0000_0000_0000), // Bit 14 active => Key E pressed

                Keycode::Z => Some(0b0000_0100_0000_0000), // Bit 10 active => Key A pressed
                Keycode::X => Some(0b0000_0000_0000_0001), // Bit  0 active => Key 0 pressed
                Keycode::C => Some(0b0000_1000_0000_0000), // Bit 11 active => Key B pressed
                Keycode::V => Some(0b1000_0000_0000_0000), // Bit 15 active => Key F pressed

                _ => None,
            };

            if let Some(i) = index {
                chip8_keys |= i;
            }
        }

        Ok(chip8_keys)
    }
}

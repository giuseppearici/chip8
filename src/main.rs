extern crate rand;
extern crate sdl2;

use std::{env, process};

use motherboard::processor::Processor;
use peripherals::audio_driver::AudioDriver;
use peripherals::cartridge_driver::CartridgeDriver;
use peripherals::display_driver::DisplayDriver;
use peripherals::input_driver::InputDriver;

mod constants;
mod logger;
mod motherboard;
mod peripherals;
mod toolchain;

fn main() {
    logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        log::error!("Error: <ROM file> missing");
        process::exit(1);
    }
    let rom_filename = &args[1];

    let sdl_context = sdl2::init().unwrap();

    let mut display_driver = DisplayDriver::new(&sdl_context);
    let mut input_driver = InputDriver::new(&sdl_context);
    let audio_driver = AudioDriver::new(&sdl_context);
    let cartridge_driver = CartridgeDriver::new(rom_filename);

    if cartridge_driver.rom_size == 0 {
        log::error!("Error: <ROM file> {} empty", rom_filename);
        process::exit(1);
    }

    let mut processor = Processor::new();
    processor.run(
        &mut display_driver,
        &mut input_driver,
        &audio_driver,
        &cartridge_driver,
    );
}

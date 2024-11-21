use crate::constants::SCREEN_SIZE;
use crate::DisplayDriver;

pub(crate) struct Screen {
    pixels: [bool; SCREEN_SIZE],
    needs_refresh: bool,
}

impl Screen {
    pub(crate) fn new() -> Self {
        Screen {
            pixels: [false; SCREEN_SIZE],
            needs_refresh: false,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.pixels = [false; SCREEN_SIZE];
        self.needs_refresh = true;
    }

    pub(crate) fn refresh(&mut self, display_driver: &mut DisplayDriver) {
        if self.needs_refresh {
            display_driver.draw(&self.pixels);
            self.needs_refresh = false;
        }
    }

    pub(crate) fn get_pixel(&self, index: usize) -> bool {
        self.pixels[index]
    }

    pub(crate) fn set_pixel(&mut self, index: usize, value: bool) {
        self.pixels[index] = value;
        self.needs_refresh = true;
    }

    pub(crate) fn get_all_pixels(&self) -> &[bool] {
        self.pixels.as_ref()
    }
}

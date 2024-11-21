use sdl2;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::constants::{
    BACKGROUND_COLOR as BG, FOREGROUND_COLOR as FG, SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub(crate) struct DisplayDriver {
    foreground_color: pixels::Color,
    background_color: pixels::Color,
    canvas: Canvas<Window>,
}

impl DisplayDriver {
    pub(crate) fn new(sdl_context: &sdl2::Sdl) -> Self {
        let foreground_color = pixels::Color::RGB(FG[0], FG[1], FG[2]);
        let background_color = pixels::Color::RGB(BG[0], BG[1], BG[2]);

        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "Chip8",
                (SCREEN_WIDTH as u32) * SCALE_FACTOR,
                (SCREEN_HEIGHT as u32) * SCALE_FACTOR,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(background_color);
        canvas.clear();
        canvas.present();

        DisplayDriver {
            canvas,
            foreground_color,
            background_color,
        }
    }

    pub(crate) fn draw(&mut self, buffer: &[bool; SCREEN_WIDTH * SCREEN_HEIGHT]) {
        // Clear canvas with background color
        self.canvas.set_draw_color(self.background_color);
        self.canvas.clear();

        // Now set draw color to foreground color, iterate through each pixel and see if it should be drawn
        for (i, pixel) in buffer.iter().enumerate() {
            if *pixel {
                self.canvas.set_draw_color(self.foreground_color);
            } else {
                self.canvas.set_draw_color(self.background_color)
            }

            // Convert our 1D array's index into a 2D (x,y) position
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            // Draw a rectangle at (x,y), scaled up by our SCALE_FACTOR value
            let rect = Rect::new(
                (x * SCALE_FACTOR) as i32,
                (y * SCALE_FACTOR) as i32,
                SCALE_FACTOR,
                SCALE_FACTOR,
            );
            self.canvas.fill_rect(rect).unwrap();
        }
        self.canvas.present();
    }
}

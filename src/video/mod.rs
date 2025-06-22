use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::time::Duration;

use crate::cpu::Cpu;

pub struct Video {
    cpu: Cpu,
}

/// Width of a real GBA screen in pixels
const GBA_VIDEO_WIDTH: u32 = 240;
/// Height of a real GBA screen in pixels
const GBA_VIDEO_HEIGHT: u32 = 160;

const VIDEO_SCALE: u32 = 6;

impl Video {
    pub fn new(cpu: Cpu) -> Self {
        Self { cpu }
    }

    pub fn initialize_screen(&self) {
        let cntrl = self.cpu.get_memory(0x4000000) as u16;
        if cntrl != 0x0403 {
            panic!("Only BG Mode 3 and Screendisplay BG2 is supported")
        }
    }

    fn get_points(&self) -> Vec<(Color, Point)> {
        let mut points = Vec::new();
        // Assuminb BG Mode 3
        for (idx, addr) in (0x06000000..=0x06012BFF_u32).step_by(2).enumerate() {
            let x = idx % GBA_VIDEO_WIDTH as usize;
            let y = idx / GBA_VIDEO_WIDTH as usize;

            let value = self.cpu.get_memory_u16(addr);
            if value != 0 {
                let r = ((value & 0x1F) as f32 / 31.0 * 255.0) as u8;
                let g = (((value >> 5) & 0x1F) as f32 / 31.0 * 255.0) as u8;
                let b = (((value >> 10) & 0x1F) as f32 / 31.0 * 255.0) as u8;

                let color = Color::RGB(r, g, b);
                let point = Point::new(x as i32, y as i32);
                points.push((color, point));
            }
        }

        points
    }

    pub fn draw(&self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "GBA Emu",
                GBA_VIDEO_WIDTH * VIDEO_SCALE,
                GBA_VIDEO_HEIGHT * VIDEO_SCALE,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        for (color, point) in self.get_points() {
            canvas.set_draw_color(color);
            let point = point.scale(VIDEO_SCALE as i32);
            let rect = Rect::new(point.x, point.y, VIDEO_SCALE, VIDEO_SCALE);
            canvas.fill_rect(rect).unwrap();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut i = 0;
        'running: loop {
            i = (i + 1) % 255;
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

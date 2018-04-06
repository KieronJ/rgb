use std::thread;
use std::time;

use sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use super::ppu::PpuShade;

pub const FRAME_TIME: f64 = (1.0 / 59.7) * 1000.0;

pub struct SdlContext {
    event_pump: sdl2::EventPump,
    canvas: sdl2::render::WindowCanvas,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,

    last_time: time::Instant,
}

impl SdlContext {
    pub fn new(width: usize, height: usize, title: &str) -> SdlContext {
        let context = sdl2::init().unwrap();
        let event_pump = context.event_pump().unwrap();
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem.window(title, width as u32, height as u32).build().unwrap();
        let canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();

        SdlContext {
            event_pump: event_pump,
            canvas: canvas,
            texture_creator: texture_creator,

            last_time: time::Instant::now(),
        }
    }

    pub fn handle_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                _ => {}
            }
        }
    }

    pub fn render(&mut self, framebuffer: &[PpuShade]) {
        let window_size = self.canvas.window().size();
        let mut texture = self.texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, window_size.0, window_size.1).unwrap();

        self.canvas.clear();

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..144 {
                for x in 0..160 {
                    let texture_address = (y * pitch) + (x * 3);
                    let framebuffer_address = (y * 160) + x;

                    let pixel_colour = match framebuffer[framebuffer_address] {
                        PpuShade::WHITE => 0xff,
                        PpuShade::LIGHT => 0xaa,
                        PpuShade::DARK  => 0x55,
                        PpuShade::BLACK => 0x00,
                    };

                    buffer[texture_address]     = pixel_colour;
                    buffer[texture_address + 1] = pixel_colour;
                    buffer[texture_address + 2] = pixel_colour;
                }
            }
        }).unwrap();

        self.canvas.copy(&texture, None, Some(Rect::new(0, 0, 160, 144))).unwrap();
        self.canvas.present();
    }

    pub fn sleep_frame(&mut self) {
        let current_time = time::Instant::now();
        
        let elapsed = current_time.duration_since(self.last_time);
		let elapsed_ms = (elapsed.as_secs() as f64 * 1000.0) + (elapsed.subsec_nanos() as f64 / 1000000.0);

        if elapsed_ms < FRAME_TIME {
            let sleep_time = (FRAME_TIME - elapsed_ms) as u64;

            if sleep_time != 0 {
                thread::sleep(time::Duration::from_millis(sleep_time));
            }
        }

        self.last_time = time::Instant::now();
    }
}
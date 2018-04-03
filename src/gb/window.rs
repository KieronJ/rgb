use sdl2;

pub struct SdlContext {
    context: sdl2::Sdl,
    event_pump: sdl2::EventPump,
    video_subsystem: sdl2::VideoSubsystem,
    canvas: sdl2::render::WindowCanvas,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
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
            context: context,
            event_pump: event_pump,
            video_subsystem: video_subsystem,
            canvas: canvas,
            texture_creator: texture_creator,
        }
    }

    pub fn handle_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                _ => {}
            }
        }
    }

    pub fn render(&mut self, framebuffer: &[u8]) {

    }
}
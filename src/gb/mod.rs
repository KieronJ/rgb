mod apu;
mod audio_system;
mod bus;
mod cartridge;
mod controller;
mod mapper;
mod ppu;
mod timer;
mod video_system;
mod z80;

use sdl2;

use self::audio_system::AudioSystem;
use self::bus::Bus;
use self::cartridge::Cartridge;
use self::mapper::Mapper;
use self::video_system::VideoSystem;
use self::z80::Z80;

pub struct Gameboy {
    cpu: Z80,
}

impl Gameboy {
    pub fn new(cartridge_filepath: &str) -> Gameboy {
        let cartridge = Cartridge::new(cartridge_filepath);
        let mapper = Mapper::new(cartridge);

        mapper.info();

        let sdl_context = sdl2::init().unwrap();

        let audio_system = AudioSystem::new(&sdl_context);
        let video_system = VideoSystem::new(&sdl_context, 160, 144, "rgb");
        let bus = Bus::new(mapper, audio_system, video_system);

        Gameboy {
            cpu: Z80::new(bus),
        }
    }

    pub fn run(&mut self) {
        self.cpu.run();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
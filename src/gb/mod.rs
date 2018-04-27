mod bus;
mod cartridge;
mod controller;
mod mapper;
mod ppu;
mod timer;
mod window;
mod z80;

use self::bus::Bus;
use self::cartridge::Cartridge;
use self::mapper::Mapper;
use self::z80::Z80;

pub struct Gameboy {
    cpu: Z80,
}

impl Gameboy {
    pub fn new(cartridge_filepath: &str) -> Gameboy {
        let cartridge = Cartridge::new(cartridge_filepath);
        let mapper = Mapper::new(cartridge);

        mapper.info();

        let bus = Bus::new(mapper);

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
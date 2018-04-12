mod bus;
mod cartridge;
mod controller;
mod ppu;
mod window;
mod z80;

use self::bus::Bus;
use self::cartridge::Cartridge;
use self::z80::Z80;

pub struct Gameboy {
    cpu: Z80,
}

impl Gameboy {
    pub fn new(cartridge_filepath: &str) -> Gameboy {
        let cartridge = Cartridge::new(cartridge_filepath);
        cartridge.print_cartridge_info();

        let mapper = cartridge.get_type().get_mapper();

        //if mapper != cartridge::CartridgeMapper::NONE {
        //    panic!("ERROR: unsupported mapper {:#?}", mapper);
        //}

        let bus = Bus::new(cartridge);

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
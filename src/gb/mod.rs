mod bus;
mod cartridge;
mod ppu;
mod window;
mod z80;

pub struct Gameboy {
    cpu: z80::Z80,
}

impl Gameboy {
    pub fn new(cartridge_filepath: &str) -> Gameboy {
        let cartridge = cartridge::Cartridge::new(cartridge_filepath);
        cartridge.print_cartridge_info();

        let mapper = cartridge.get_type().get_mapper();

        if mapper != cartridge::CartridgeMapper::NONE {
            panic!("ERROR: unsupported mapper {:#?}", mapper);
        }

        let bus = bus::Bus::new(cartridge);

        Gameboy {
            cpu: z80::Z80::new(bus),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.run();
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn cpu(&self) -> &z80::Z80 {
        &self.cpu
    }
}
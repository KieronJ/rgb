use super::cartridge;
use super::ppu;

const BOOTROM: &'static [u8] = include_bytes!("../../bootrom/DMG_ROM.bin");

pub struct Bus {
    latch: u8,

    bootrom: Box<[u8]>,
    bootrom_enabled: bool,

    cartridge: cartridge::Cartridge,

    ppu: ppu::Ppu,

    zero_page: Box<[u8]>,
}

impl Bus {
    pub fn new(cartridge: cartridge::Cartridge) -> Bus {
        Bus {
            latch: 0,

            bootrom: Box::from(BOOTROM),
            bootrom_enabled: true,

            cartridge: cartridge,

            ppu: ppu::Ppu::new(),

            zero_page: vec![0u8; 0x7f].into_boxed_slice(),
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        let address = address as usize;

        if address < 0x0100 && self.bootrom_enabled {
            self.latch = self.bootrom[address];
        } 
        
        else if address < 0x0100 {
            self.latch = self.cartridge.read_rom(address);
        } 
        
        else if address >= 0x0100 && address < 0x8000 {
            self.latch = self.cartridge.read_rom(address);
        } 
        
        else if address >= 0x8000 && address < 0xa000 {
            self.latch = self.ppu.vram_read(address as u16);
        } 
        
        else if address >= 0xfe00 && address < 0xfea0 {
            self.latch = self.ppu.vram_read(address as u16);
        }

        else if address >= 0xff00 && address < 0xff80 {
            self.latch = self.io_read(address as u16);
        } 
        
        else if address >= 0xff80 && address < 0xffff {
            self.latch = self.zero_page[address - 0xff80];
        } 
        
        else {
            panic!("ERROR: read from unknown address 0x{:04x}", address)
        }

        self.latch
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let address = address as usize;

        self.latch = value;
        let latch = self.latch;

        if address >= 0x8000 && address < 0xa000 {
            self.ppu.vram_write(address as u16, latch);
        } 
        
        else if address >= 0xfe00 && address < 0xfea0 {
            self.ppu.vram_write(address as u16, latch)
        }

        else if address >= 0xff00 && address < 0xff80 {
            self.io_write(address as u16, latch);
        } 
        
        else if address >= 0xff80 && address < 0xffff {
            self.zero_page[address - 0xff80] = latch;
        } 
        
        else {
            panic!("ERROR: write to unknown address 0x{:04x}", address)
        }
    }

    pub fn io_read(&mut self, address: u16) -> u8 {
        match address {
            0xff44 => self.latch = self.ppu.ly_read(),
            _ => println!("WARN: read from unimplemented i/o register 0x{:04x}", address),
        }

        self.latch
    }

    pub fn io_write(&mut self, address: u16, value: u8) {
        match address {
            0xff44 => (),
            0xff50 => self.bootrom_enabled = false,
            _ => println!("WARN: write to unimplemented i/o register 0x{:04x}", address),
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick();
    }
}
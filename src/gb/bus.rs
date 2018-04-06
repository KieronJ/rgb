use super::cartridge;
use super::ppu;

const BOOTROM: &'static [u8] = include_bytes!("../../bootrom/DMG_ROM.bin");

bitflags! {
    pub struct InterruptEnable: u8 {
        const JOYPAD   = 0b00010000;
        const SERIAL   = 0b00001000;
        const TIMER    = 0b00000100;
        const LCD_STAT = 0b00000010;
        const VBLANK   = 0b00000001;
    }
}

pub struct Bus {
    latch: u8,

    bootrom: Box<[u8]>,
    bootrom_enabled: bool,

    cartridge: cartridge::Cartridge,

    ppu: ppu::Ppu,

    work_ram: Box<[u8]>,

    serial_buffer: u8,
    serial_control: u8,

    high_ram: Box<[u8]>,

    ie: InterruptEnable,
}

impl Bus {
    pub fn new(cartridge: cartridge::Cartridge) -> Bus {
        Bus {
            latch: 0,

            bootrom: Box::from(BOOTROM),
            bootrom_enabled: true,

            cartridge: cartridge,

            ppu: ppu::Ppu::new(),

            work_ram: vec![0; 0x2000].into_boxed_slice(),

            serial_buffer: 0,
            serial_control: 0,

            high_ram: vec![0; 0x7f].into_boxed_slice(),

            ie: InterruptEnable::empty(),
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
        
        else if address >= 0xc000 && address < 0xe000 {
            self.latch = self.work_ram[address - 0xc000];
        }

        else if address >= 0xe000 && address < 0xfe00 {
            self.latch = self.work_ram[address - 0xe000];
        }

        else if address >= 0xfe00 && address < 0xfea0 {
            self.latch = self.ppu.vram_read(address as u16);
        }

        else if address >= 0xfea0 && address < 0xff00 {
            println!("WARN: read from unusable memory");
        }

        else if address >= 0xff00 && address < 0xff80 {
            self.latch = self.io_read(address as u16);
        }
        
        else if address >= 0xff80 && address < 0xffff {
            self.latch = self.high_ram[address - 0xff80];
        }

        else if address == 0xffff {
            self.latch = self.ie.bits()
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

        if address < 0x8000 {
            self.cartridge.write_rom(address, latch);
        }

        else if address >= 0x8000 && address < 0xa000 {
            self.ppu.vram_write(address as u16, latch);
        }
        
        else if address >= 0xc000 && address < 0xe000 {
            self.work_ram[address - 0xc000] = latch;
        }

        else if address >= 0xe000 && address < 0xfe00 {
            self.work_ram[address - 0xe000] = latch;
        }

        else if address >= 0xfe00 && address < 0xfea0 {
            self.ppu.vram_write(address as u16, latch)
        }

        else if address >= 0xfea0 && address < 0xff00 {
            println!("WARN: write to unusable memory");
        }

        else if address >= 0xff00 && address < 0xff80 {
            self.io_write(address as u16, latch);
        }
        
        else if address >= 0xff80 && address < 0xffff {
            self.high_ram[address - 0xff80] = latch;
        }

        else if address == 0xffff {
            self.ie = InterruptEnable::from_bits_truncate(latch);
        }

        else {
            panic!("ERROR: write to unknown address 0x{:04x}", address)
        }
    }

    pub fn io_read(&mut self, address: u16) -> u8 {
        self.latch = match address {
            0xff01 => self.serial_buffer,
            0xff02 => self.serial_control,
            0xff40 => self.ppu.lcdc_read(),
            0xff42 => self.ppu.scy_read(),
            0xff44 => self.ppu.ly_read(),
            0xff47 => self.ppu.bgp_read(),
            _ => { println!("WARN: read from unimplemented i/o register 0x{:04x}", address); self.latch },
        };

        self.latch
    }

    pub fn io_write(&mut self, address: u16, value: u8) {
        match address {
            0xff01 => self.serial_buffer = value,
            0xff02 => {
                self.serial_control = value;

                if self.serial_control == 0x81 {
                    println!("SERIAL TRANSFER: {}", self.serial_buffer as char);
                }
            }
            0xff40 => self.ppu.lcdc_write(value),
            0xff42 => self.ppu.scy_write(value),
            0xff44 => (),
            0xff47 => self.ppu.bgp_write(value),
            0xff50 => self.bootrom_enabled = false,
            _ => println!("WARN: write to unimplemented i/o register 0x{:04x}", address),
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick();
    }
}
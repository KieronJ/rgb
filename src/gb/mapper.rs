use super::cartridge::{Cartridge, CartridgeMapper};

pub trait Mapper {
    fn read_rom(&mut self, address: u16) -> u8;
    fn read_ram(&mut self, address: u16) -> u8;

    fn write_rom(&mut self, address: u16, value: u8);
    fn write_ram(&mut self, address: u16, value: u8);

    fn info(&self);
}

impl Mapper {
    pub fn new(cartridge: Cartridge) -> Box<Mapper + Send> {
        match cartridge.get_type().get_mapper() {
            CartridgeMapper::NONE => Box::new(MapperNone::new(cartridge)),
            CartridgeMapper::MBC1 => Box::new(MapperMBC1::new(cartridge)),
            CartridgeMapper::MBC3 => Box::new(MapperMBC3::new(cartridge)),
            _ => panic!("ERROR: unsupported mapper"),
        }
    }
}

pub struct MapperNone {
    cartridge: Cartridge,
}

impl MapperNone {
    pub fn new(cartridge: Cartridge) -> MapperNone {
        MapperNone {
            cartridge: cartridge,
        }
    }
}

impl Mapper for MapperNone {
    fn read_rom(&mut self, address: u16) -> u8 {
        self.cartridge.read_rom(address as usize)
    }

    fn read_ram(&mut self, address: u16) -> u8 {
        self.cartridge.read_ram(address as usize)
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        self.cartridge.write_rom(address as usize, value);
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        self.cartridge.write_ram(address as usize, value);
    }

    fn info(&self) {
        self.cartridge.info();
    }
}

#[derive(PartialEq)]
enum MBC1BankingMode {
    ROM,
    RAM,
}

pub struct MapperMBC1 {
    cartridge: Cartridge,
    ram_enable: bool,
    rom_bank: u8,
    ram_bank: u8,
    banking_mode: MBC1BankingMode,
}

impl MapperMBC1 {
    pub fn new(cartridge: Cartridge) -> MapperMBC1 {
        MapperMBC1 {
            cartridge: cartridge,
            ram_enable: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: MBC1BankingMode::ROM,
        }
    }

    fn rom_bank_lower(&self) -> u8 {
        if self.banking_mode == MBC1BankingMode::ROM {
            (self.ram_bank & 0x03) << 5
        } else {
            0
        }
    }

    fn rom_bank_upper(&self) -> u8 {
        let mut rom_bank = self.rom_bank & 0x1f;

        if self.banking_mode == MBC1BankingMode::ROM {
            rom_bank |= (self.ram_bank & 0x03) << 5;
        }

        rom_bank
    }

    fn ram_bank(&self) -> u8 {
        let mut ram_bank = 0;

        if self.banking_mode == MBC1BankingMode::RAM {
            ram_bank = self.ram_bank & 0x03;
        }

        ram_bank
    }
}

impl Mapper for MapperMBC1 {
    fn read_rom(&mut self, address: u16) -> u8 {
        let address = address as usize;
        let bank;

        if address >= 0x4000 {
            bank = self.rom_bank_upper() as usize;
        } else {
            bank = self.rom_bank_lower() as usize;
        }
        
        let rom_size = self.cartridge.get_rom_size().unwrap() - 1;
        let bank_address = (bank * 0x4000) | (address & 0x3fff);
        self.cartridge.read_rom(bank_address & rom_size)
    }

    fn read_ram(&mut self, address: u16) -> u8 {
        let address = address as usize;
        let bank = self.ram_bank() as usize;

        let ram_size = self.cartridge.get_ram_size().unwrap() - 1;
        let bank_address = (bank * 0x2000) | (address & 0x1fff);

        if self.ram_enable {
            self.cartridge.read_ram(bank_address & ram_size)
        } else {
            0xff
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x1fff => {
                self.ram_enable = (value & 0x0a) == 0x0a;
            },
            0x2000...0x3fff => {
                self.rom_bank = value & 0x1f;

                if self.rom_bank & 0x1f == 0 {
                    self.rom_bank = 0x01;
                }
            },
            0x4000...0x5fff => {
                self.ram_bank = value & 0x03;
            },
            0x6000...0x7fff => {
                self.banking_mode = match (value & 0x01) != 0 {
                    true => MBC1BankingMode::RAM,
                    false => MBC1BankingMode::ROM,
                };
            },
            _ => unreachable!(),
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        let address = address as usize;
        let bank = self.ram_bank() as usize;

        let ram_size = self.cartridge.get_ram_size().unwrap().wrapping_sub(1);
        let bank_address = (bank * 0x2000) | (address & 0x1fff);

        if self.ram_enable {
            self.cartridge.write_ram(bank_address & ram_size, value);
        }
    }

    fn info(&self) {
        self.cartridge.info();
    }
}

#[derive(PartialEq)]
enum MBC3BankingMode {
    ROM,
    RAM,
}

pub struct MapperMBC3 {
    cartridge: Cartridge,
    ram_enable: bool,
    rom_bank: u8,
    ram_bank: u8,
    banking_mode: MBC1BankingMode,
}

impl MapperMBC3 {
    pub fn new(cartridge: Cartridge) -> MapperMBC1 {
        MapperMBC1 {
            cartridge: cartridge,
            ram_enable: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: MBC1BankingMode::ROM,
        }
    }

    fn rom_bank_lower(&self) -> u8 {
        if self.banking_mode == MBC1BankingMode::ROM {
            (self.ram_bank & 0x03) << 5
        } else {
            0
        }
    }

    fn rom_bank_upper(&self) -> u8 {
        let mut rom_bank = self.rom_bank & 0x7f;

        if self.banking_mode == MBC1BankingMode::ROM {
            rom_bank |= (self.ram_bank & 0x03) << 5;
        }

        rom_bank
    }

    fn ram_bank(&self) -> u8 {
        let mut ram_bank = 0;

        if self.banking_mode == MBC1BankingMode::RAM {
            ram_bank = self.ram_bank & 0x03;
        }

        ram_bank
    }
}

impl Mapper for MapperMBC3 {
    fn read_rom(&mut self, address: u16) -> u8 {
        let address = address as usize;
        let bank;

        if address >= 0x4000 {
            bank = self.rom_bank_upper() as usize;
        } else {
            bank = self.rom_bank_lower() as usize;
        }
        
        let rom_size = self.cartridge.get_rom_size().unwrap() - 1;
        let bank_address = (bank * 0x4000) | (address & 0x3fff);
        self.cartridge.read_rom(bank_address & rom_size)
    }

    fn read_ram(&mut self, address: u16) -> u8 {
        let address = address as usize;
        let bank = self.ram_bank() as usize;

        let ram_size = self.cartridge.get_ram_size().unwrap() - 1;
        let bank_address = (bank * 0x2000) | (address & 0x1fff);

        if self.ram_enable {
            self.cartridge.read_ram(bank_address & ram_size)
        } else {
            0xff
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x1fff => {
                self.ram_enable = (value & 0x0a) == 0x0a;
            },
            0x2000...0x3fff => {
                self.rom_bank = value & 0x1f;

                if self.rom_bank & 0x1f == 0 {
                    self.rom_bank = 0x01;
                }
            },
            0x4000...0x5fff => {
                self.ram_bank = value & 0x03;
            },
            0x6000...0x7fff => {
                self.banking_mode = match (value & 0x01) != 0 {
                    true => MBC1BankingMode::RAM,
                    false => MBC1BankingMode::ROM,
                };
            },
            _ => unreachable!(),
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        let address = address as usize;
        let bank = self.ram_bank() as usize;

        let ram_size = self.cartridge.get_ram_size().unwrap().wrapping_sub(1);
        let bank_address = (bank * 0x2000) | (address & 0x1fff);

        if self.ram_enable {
            self.cartridge.write_ram(bank_address & ram_size, value);
        }
    }

    fn info(&self) {
        self.cartridge.info();
    }
}
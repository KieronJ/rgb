use std::fs::File;
use std::io::Read;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CartridgeMapper {
    NONE,
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC4,
    MBC5,
    POCKETCAMERA,
    TAMA5,
    HUC3,
    HUC1,
    INVALID
}

#[derive(Debug)]
pub struct CartridgeType {
    mapper: CartridgeMapper,
    ram: bool,
    battery: bool,
    timer: bool,
    rumble: bool,
}

impl CartridgeType {
    pub fn new(mapper: CartridgeMapper, ram: bool, battery: bool, timer: bool, rumble: bool) -> CartridgeType {
        CartridgeType {
            mapper,
            ram,
            battery,
            timer,
            rumble
        }
    }

    pub fn get_mapper(&self) -> CartridgeMapper {
        self.mapper
    }
}

#[derive(Debug)]
pub enum CartridgeLanguage {
    JAPANESE,
    ENGLISH,
    INVALID
}

pub struct Cartridge {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
}

impl Cartridge {
    pub fn new(filepath: &str) -> Cartridge {
        let mut f = File::open(filepath).unwrap();
        let mut buffer = Vec::new();

        let _ = f.read_to_end(&mut buffer);

        let ram_size = match buffer[0x149] {
            0x00 => 0x0,
            0x01 => 0x800,
            0x02 => 0x2000,
            0x03 => 0x8000,
            0x04 => 0x20000,
            0x05 => 0x10000,
            _    => panic!("ERROR: invalid cartridge ram size")
        };

        Cartridge {
            rom: buffer.into_boxed_slice(),
            ram: vec![0u8; ram_size].into_boxed_slice(),
        }
    }

    pub fn read_rom(&self, address: usize) -> u8 {
        if address < self.get_rom_size().unwrap() {
            self.rom[address]
        } else {
            panic!("ERROR: read from out of bounds rom address")
        }
    }

    pub fn read_ram(&self, address: usize) -> u8 {
        if address < self.get_ram_size().unwrap() {
            self.ram[address]
        } else {
            println!("WARN: read from out of bounds ram address");
            0xff
        }
    }

    pub fn write_rom(&mut self, _: usize, _: u8) {
        println!("WARN: write to rom")
    }

    pub fn write_ram(&mut self, address: usize, value: u8) {
        if address < self.get_ram_size().unwrap() {
            self.ram[address] = value
        } else {
            println!("WARN: write to out of bounds ram address");
        }
    }

    pub fn get_title(&self) -> String {
        let mut title = String::new();
        let mut title_length = 0;

        let mut character = self.rom[0x134] as char;

        while character != '\0' {
            title.push(character);
            title_length += 1;

            character = self.rom[0x134 + title_length] as char;
        }

        title
    }

    pub fn get_filesize(&self) -> usize {
        self.rom.len()
    }

    pub fn get_rom_size(&self) -> Result<usize, &str> {
        match self.rom[0x148] {
            0x00 => Ok(0x8000),
            0x01 => Ok(0x10000),
            0x02 => Ok(0x20000),
            0x03 => Ok(0x40000),
            0x04 => Ok(0x80000),
            0x05 => Ok(0x100000),
            0x06 => Ok(0x200000),
            0x07 => Ok(0x400000),
            0x08 => Ok(0x800000),
            _ => Err("ERROR: invalid cartridge ROM size")
        }
    }

    pub fn get_rom_banks(&self) -> Result<usize, &str> {
        match self.rom[0x148] {
            0x00 => Ok(2),
            0x01 => Ok(4),
            0x02 => Ok(8),
            0x03 => Ok(16),
            0x04 => Ok(32),
            0x05 => Ok(64),
            0x06 => Ok(128),
            0x07 => Ok(256),
            0x08 => Ok(512),
            _ => Err("ERROR: invalid cartridge ROM size")
        }
    }

    pub fn get_ram_size(&self) -> Result<usize, &str> {
        match self.rom[0x149] {
            0x00 => Ok(0),
            0x01 => Ok(1),
            0x02 => Ok(1),
            0x03 => Ok(4),
            0x04 => Ok(16),
            0x05 => Ok(8),
            _ => Err("ERROR: invalid cartridge RAM size")
        }
    }

    pub fn get_ram_banks(&self) -> Result<usize, &str> {
        match self.rom[0x149] {
            0x00 => Ok(0),
            0x01 => Ok(0x800),
            0x02 => Ok(0x2000),
            0x03 => Ok(0x8000),
            0x04 => Ok(0x20000),
            0x05 => Ok(0x10000),
            _ => Err("ERROR: invalid cartridge RAM size")
        }
    }

    pub fn has_ram(&self) -> bool {
        self.get_ram_size().unwrap() > 0
    }

    pub fn get_language(&self) -> CartridgeLanguage {
        match self.rom[0x14a] {
            0x00 => CartridgeLanguage::JAPANESE,
            0x01 => CartridgeLanguage::ENGLISH,
            _  => CartridgeLanguage::INVALID
        }
    }

    pub fn get_type(&self) -> CartridgeType {
        match self.rom[0x147] {
            0x00 => CartridgeType::new(CartridgeMapper::NONE, false, false, false, false),
            0x01...0x03 => CartridgeType::new(CartridgeMapper::MBC1, false, false, false, false),
            0x13 => CartridgeType::new(CartridgeMapper::MBC3, false, false, false, false),
            _    => CartridgeType::new(CartridgeMapper::INVALID, false, false, false, false)
        }
    }

    pub fn info(&self) {
        println!("Filesize: {}kb", self.get_filesize() / 1024);
        println!("Title: {}", self.get_title());
        println!("Type: {:#?}", self.get_type());

        let rom_size = self.get_rom_size().unwrap();
        let rom_banks = self.get_rom_banks().unwrap();
        println!("ROM Size: {}kb ({} banks)", rom_size / 0x400, rom_banks);

        let ram_size = self.get_ram_size().unwrap();
        let ram_banks = self.get_ram_banks().unwrap();
        println!("RAM Size: {}kb ({} banks)", ram_size / 0x400, ram_banks);

        println!("Language: {:#?}", self.get_language());
    }
}
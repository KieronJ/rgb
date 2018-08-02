use super::apu::Apu;
use super::audio_system::AudioSystem;
use super::mapper::Mapper;
use super::ppu::Ppu;
use super::timer::Timer;
use super::video_system::VideoSystem;

const BOOTROM: &'static [u8] = include_bytes!("../../bootrom/DMG_ROM.bin");

bitflags! {
    pub struct Interrupts: u8 {
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

    mapper: Box<Mapper + Send>,

    apu: Apu,
    ppu: Ppu,

    work_ram: Box<[u8]>,

    serial_buffer: u8,
    serial_control: u8,

    timer: Timer,

    high_ram: Box<[u8]>,

    interrupt_enable: Interrupts,
    interrupt_flag: Interrupts,
}

impl Bus {
    pub fn new(mapper: Box<Mapper + Send>, audio_system: AudioSystem, video_system: VideoSystem) -> Bus {
        Bus {
            latch: 0,

            bootrom: Box::from(BOOTROM),
            bootrom_enabled: true,

            mapper: mapper,

            apu: Apu::new(audio_system),
            ppu: Ppu::new(video_system),

            work_ram: vec![0; 0x2000].into_boxed_slice(),

            serial_buffer: 0,
            serial_control: 0,

            timer: Timer::new(),

            high_ram: vec![0; 0x7f].into_boxed_slice(),

            interrupt_enable: Interrupts::empty(),
            interrupt_flag: Interrupts::empty(),
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        if address < 0x0100 && self.bootrom_enabled {
            self.latch = self.bootrom[address as usize];
        }
        
        else if address < 0x0100 {
            self.latch = self.mapper.read_rom(address);
        }
        
        else if address >= 0x0100 && address < 0x8000 {
            self.latch = self.mapper.read_rom(address);
        }
        
        else if address >= 0x8000 && address < 0xa000 {
            self.latch = self.ppu.vram_read(address as u16);
        }
        
        else if address >= 0xa000 && address < 0xc000 {
            self.latch = self.mapper.read_ram(address);
        }

        else if address >= 0xc000 && address < 0xe000 {
            self.latch = self.work_ram[address as usize - 0xc000];
        }

        else if address >= 0xe000 && address < 0xfe00 {
            self.latch = self.work_ram[address as usize - 0xe000];
        }

        else if address >= 0xfe00 && address < 0xfea0 {
            self.latch = self.ppu.vram_read(address);
        }

        else if address >= 0xfea0 && address < 0xff00 {
            println!("WARN: read from unusable memory");
            self.latch = 0x00;
        }

        else if address >= 0xff00 && address < 0xff80 {
            self.latch = self.io_read(address);
        }
        
        else if address >= 0xff80 && address < 0xffff {
            self.latch = self.high_ram[address as usize - 0xff80];
        }

        else if address == 0xffff {
            self.latch = self.interrupt_enable.bits()
        }

        else {
            panic!("ERROR: read from unknown address 0x{:04x}", address)
        }

        self.latch
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.latch = value;
        let latch = self.latch;

        if address < 0x8000 {
            self.mapper.write_rom(address, latch);
        }

        else if address >= 0x8000 && address < 0xa000 {
            self.ppu.vram_write(address, latch);
        }
        
        else if address >= 0xa000 && address < 0xc000 {
            self.mapper.write_ram(address, latch);
        }

        else if address >= 0xc000 && address < 0xe000 {
            self.work_ram[address as usize - 0xc000] = latch;
        }

        else if address >= 0xe000 && address < 0xfe00 {
            self.work_ram[address as usize - 0xe000] = latch;
        }

        else if address >= 0xfe00 && address < 0xfea0 {
            self.ppu.vram_write(address, latch)
        }

        else if address >= 0xfea0 && address < 0xff00 {

        }

        else if address >= 0xff00 && address < 0xff80 {
            self.io_write(address, latch);
        }
        
        else if address >= 0xff80 && address < 0xffff {
            self.high_ram[address as usize - 0xff80] = latch;
        }

        else if address == 0xffff {
            self.interrupt_enable = Interrupts::from_bits_truncate(latch);
        }

        else {
            panic!("ERROR: write to unknown address 0x{:04x}", address)
        }
    }

    fn io_read(&mut self, address: u16) -> u8 {
        self.latch = match address {
            0xff00 => self.ppu.controller_read(),
            0xff01 => self.serial_buffer,
            0xff02 => self.serial_control,
            0xff04 => self.timer.div_read(),
            0xff05 => self.timer.tima_read(),
            0xff06 => self.timer.tma_read(),
            0xff07 => self.timer.tac_read(),
            0xff0f => self.process_interrupt_flag(),
            0xff40 => self.ppu.lcdc_read(),
            0xff41 => self.ppu.stat_read(),
            0xff42 => self.ppu.scy_read(),
            0xff43 => self.ppu.scx_read(),
            0xff44 => self.ppu.ly_read(),
            0xff45 => self.ppu.lyc_read(),
            0xff47 => self.ppu.bgp_read(),
            0xff48 => self.ppu.obp1_read(),
            0xff49 => self.ppu.obp2_read(),
            _ => { println!("WARN: read from unimplemented i/o register 0x{:04x}", address); 0xff },
        };

        self.latch
    }

    fn io_write(&mut self, address: u16, value: u8) {
        match address {
            0xff00 => self.ppu.controller_write(value),
            0xff01 => self.serial_buffer = value,
            0xff02 => {
                self.serial_control = value;

                if self.serial_control == 0x81 {
                    println!("SERIAL TRANSFER: {}", self.serial_buffer as char);
                }
            },
            0xff04 => self.timer.div_write(value),
            0xff05 => self.timer.tima_write(value),
            0xff06 => self.timer.tma_write(value),
            0xff07 => self.timer.tac_write(value),
            0xff0f => self.interrupt_flag = Interrupts::from_bits_truncate(value),
            0xff11 => self.apu.nr11_write(value),
            0xff12 => self.apu.nr12_write(value),
            0xff13 => self.apu.nr13_write(value),
            0xff14 => self.apu.nr14_write(value),
            0xff24 => self.apu.nr50_write(value),
            0xff25 => self.apu.nr51_write(value),
            0xff26 => self.apu.nr52_write(value),
            0xff40 => self.ppu.lcdc_write(value),
            0xff41 => self.ppu.stat_write(value),
            0xff42 => self.ppu.scy_write(value),
            0xff43 => self.ppu.scx_write(value),
            0xff44 => (),
            0xff45 => self.ppu.lyc_write(value),
            0xff46 => {
                let data_address = (value as u16) << 8;

                for i in 0x00..0xa0 {
                    let data = self.read(data_address + i);
                    self.write(0xfe00 + i, data);
                }
            }
            0xff47 => self.ppu.bgp_write(value),
            0xff48 => self.ppu.obp1_write(value),
            0xff49 => self.ppu.obp2_write(value),
            0xff50 => self.bootrom_enabled = false,
            _ => println!("WARN: write to unimplemented i/o register 0x{:04x}", address),
        }
    }

    fn process_interrupt_flag(&mut self) -> u8 {
        if self.ppu.controller().get_interrupt_status() {
            self.interrupt_flag.set(Interrupts::JOYPAD, true);
        }

        if self.timer.get_interrupt_status() {
            self.interrupt_flag.set(Interrupts::TIMER, true);
        }

        if self.ppu.get_lcdc_status() {
            self.interrupt_flag.set(Interrupts::LCD_STAT, true);
        }

        if self.ppu.get_vblank_status() {
            self.interrupt_flag.set(Interrupts::VBLANK, true);
        }

        self.interrupt_flag.bits()
    }

    pub fn tick(&mut self) {
        for _ in 0..4 {
            self.apu.tick();
            self.ppu.tick();
            self.timer.tick();
        }
    }
}
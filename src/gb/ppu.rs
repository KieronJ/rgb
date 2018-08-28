use super::controller::Controller;
use super::video_system::VideoSystem;

pub const PPU_DISPLAY_WIDTH: usize = 160;
pub const PPU_DISPLAY_HEIGHT: usize = 144;

pub const PPU_OAM_CLOCKS: usize = 80;
pub const PPU_VRAM_CLOCKS: usize = 172;
pub const PPU_HBLANK_CLOCKS: usize = 204;
pub const PPU_VBLANK_CLOCKS: usize = 456;

pub const PPU_VBLANK_START: usize = 144;
pub const PPU_VBLANK_END: usize = 154;

#[derive(Clone, Copy)]
pub enum PpuMode {
    OAM,
    VRAM,
    HBLANK,
    VBLANK,
}

#[derive(Clone, Copy)]
enum PpuSpriteSize {
    NORMAL,
    TALL,
}

pub struct PpuControl {
    lcd_enable: bool,
    //...
    window_enable: bool,
    background_pattern_address: bool,
    background_tile_address: bool,
    sprite_size: PpuSpriteSize,
    sprite_enable: bool,
    background_enable: bool,
}

impl PpuControl {
    pub fn new() -> PpuControl {
        PpuControl {
            lcd_enable: false,
            window_enable: false,
            background_pattern_address: false,
            background_tile_address: false,
            sprite_size: PpuSpriteSize::NORMAL,
            sprite_enable: false,
            background_enable: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.lcd_enable as u8)                 << 7 |
        (self.window_enable as u8)              << 5 |
        (self.background_pattern_address as u8) << 4 |
        (self.background_tile_address as u8)    << 3 |
        (self.sprite_size as u8)                << 2 |
        (self.sprite_enable as u8)              << 1 |
        self.background_enable as u8
    }

    pub fn write(&mut self, value: u8) {
        self.lcd_enable = (value & 0x80) != 0;
        self.window_enable = (value & 0x20) != 0;
        self.background_pattern_address = (value & 0x10) != 0;
        self.background_tile_address = (value & 0x08) != 0;

        self.sprite_size = match (value & 0x04) != 0 {
            false => PpuSpriteSize::NORMAL,
            true => PpuSpriteSize::TALL,
        };

        self.sprite_enable = (value & 0x02) != 0;
        self.background_enable = (value & 0x01) != 0;
    }
}

pub struct PpuStatus {
    coincidence_interrupt_enable: bool,
    oam_interrupt_enable: bool,
    vblank_interrupt_enable: bool,
    hblank_interrupt_enable: bool,
    coincidence: bool,
}

impl PpuStatus {
    pub fn new() -> PpuStatus {
        PpuStatus {
            coincidence_interrupt_enable: false,
            oam_interrupt_enable: false,
            vblank_interrupt_enable: false,
            hblank_interrupt_enable: false,
            coincidence: false,
        }
    }

    pub fn read(&self, mode: PpuMode) -> u8 {
        (self.coincidence_interrupt_enable as u8) << 6 |
        (self.oam_interrupt_enable as u8)         << 5 |
        (self.vblank_interrupt_enable as u8)      << 4 |
        (self.hblank_interrupt_enable as u8)      << 3 |
        (self.coincidence as u8)                  << 2 |
        match mode {
            PpuMode::HBLANK => 0b00,
            PpuMode::VBLANK => 0b01,
            PpuMode::OAM => 0b10,
            PpuMode::VRAM => 0b11,
        }
    }

    pub fn write(&mut self, value: u8) {
        self.coincidence_interrupt_enable = (value & 0x40) != 0;
        self.oam_interrupt_enable = (value & 0x20) != 0;
        self.vblank_interrupt_enable = (value & 0x10) != 0;
        self.hblank_interrupt_enable = (value & 0x08) != 0;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PpuShade {
    WHITE,
    LIGHT,
    DARK,
    BLACK,
}

impl PpuShade {
    pub fn from_u8(value: u8) -> PpuShade {
        match value & 0x03 {
            0b00 => PpuShade::WHITE,
            0b01 => PpuShade::LIGHT,
            0b10 => PpuShade::DARK,
            0b11 => PpuShade::BLACK,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Copy)]
pub struct PpuPalette {
    colour3: PpuShade,
    colour2: PpuShade,
    colour1: PpuShade,
    colour0: PpuShade,
}

impl PpuPalette {
    pub fn new() -> PpuPalette {
        PpuPalette {
            colour3: PpuShade::WHITE,
            colour2: PpuShade::WHITE,
            colour1: PpuShade::WHITE,
            colour0: PpuShade::WHITE,
        }
    }

    pub fn read(&self) -> u8 {
        ((self.colour3 as u8) << 6) |
        ((self.colour2 as u8) << 4) |
        ((self.colour1 as u8) << 2) |
          self.colour0 as u8
    }

    pub fn write(&mut self, value: u8) {
        self.colour3 = PpuShade::from_u8((value >> 6) & 0x03);
        self.colour2 = PpuShade::from_u8((value >> 4) & 0x03);
        self.colour1 = PpuShade::from_u8((value >> 2) & 0x03);
        self.colour0 = PpuShade::from_u8(value & 0x03);
    }
}

pub struct Ppu {
    video_system: VideoSystem,
    controller: Controller,

    framebuffer: Box<[PpuShade]>,

    latch: u8,

    control: PpuControl,
    status: PpuStatus,

    scroll_y: u8,
    scroll_x: u8,

    ly_compare: u8,

    window_y: u8,
    window_x: u8,

    background_palette: PpuPalette,

    sprite_palette_0: PpuPalette,
    sprite_palette_1: PpuPalette,

    tile_ram: Box<[u8]>,
    background_ram: Box<[u8]>,
    sprite_oam: Box<[u8]>,

    mode: PpuMode,
    mode_clocks: usize,
    scanline: usize,

    vblank: bool,
    stat_interrupt: bool,
}

impl Ppu {
    pub fn new(video_system: VideoSystem) -> Ppu {
        Ppu {
            video_system: video_system,
            controller: Controller::new(),

            framebuffer: vec![PpuShade::WHITE; PPU_DISPLAY_WIDTH * PPU_DISPLAY_HEIGHT].into_boxed_slice(),

            latch: 0,

            control: PpuControl::new(),
            status: PpuStatus::new(),

            scroll_y: 0,
            scroll_x: 0,

            ly_compare: 0,

            window_y: 0,
            window_x: 0,

            background_palette: PpuPalette::new(),

            sprite_palette_0: PpuPalette::new(),
            sprite_palette_1: PpuPalette::new(),

            tile_ram: vec![0; 0x1800].into_boxed_slice(),
            background_ram: vec![0; 0x800].into_boxed_slice(),
            sprite_oam: vec![0; 0xa0].into_boxed_slice(),

            mode: PpuMode::OAM,
            mode_clocks: 0,
            scanline: 0,

            vblank: false,
            stat_interrupt: false,
        }
    }

    pub fn vram_read(&mut self, address: u16) -> u8 {
        let address = address as usize;

        if address >= 0x8000 && address < 0x9800 {
            self.latch = self.tile_ram[address - 0x8000];
        }

        else if address >= 0x9800 && address < 0xa000 {
            self.latch = self.background_ram[address - 0x9800];
        }

        else if address >= 0xfe00 && address < 0xfea0 {
            self.latch = self.sprite_oam[address - 0xfe00];
        }

        else {
            panic!("ERROR: vram read from unknown address 0x{:04x}", address)
        }

        self.latch
    }

    pub fn vram_write(&mut self, address: u16, value: u8) {
        let address = address as usize;

        self.latch = value;

        if address >= 0x8000 && address < 0x9800 {
            self.tile_ram[address - 0x8000] = self.latch;
        }

        else if address >= 0x9800 && address < 0xa000 {
            self.background_ram[address - 0x9800] = self.latch;
        }

        else if address >= 0xfe00 && address < 0xfea0 {
            self.sprite_oam[address - 0xfe00] = self.latch;
        }

        else {
            panic!("ERROR: vram write to unknown address 0x{:04x}", address)
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.mode_clocks += cycles;

        match self.mode {
            PpuMode::OAM => {
                if self.mode_clocks >= PPU_OAM_CLOCKS {
                    self.mode_clocks -= PPU_OAM_CLOCKS;
                    self.mode = PpuMode::VRAM;
                }
            },

            PpuMode::VRAM => {
                if self.mode_clocks >= PPU_VRAM_CLOCKS {
                    self.mode_clocks -= PPU_VRAM_CLOCKS;
                    self.mode = PpuMode::HBLANK;

                    if self.status.hblank_interrupt_enable {
                        self.stat_interrupt = true;
                    }

                    if self.control.lcd_enable {
                        self.render_scanline();
                    }
                }
            },

            PpuMode::HBLANK => {
                if self.mode_clocks >= PPU_HBLANK_CLOCKS {
                    self.mode_clocks -= PPU_HBLANK_CLOCKS;
                    self.scanline += 1;

                    if self.scanline == PPU_VBLANK_START {
                        self.mode = PpuMode::VBLANK;
                        self.vblank = true;

                        if self.status.vblank_interrupt_enable {
                            self.stat_interrupt = true;
                        }

                        self.video_system.handle_events(&mut self.controller);
                        self.video_system.render(&self.framebuffer);
                        self.video_system.sync();
                    } else {
                        self.mode = PpuMode::OAM;

                        if self.status.oam_interrupt_enable {
                            self.stat_interrupt = true;
                        }
                    }
                }

                if self.scanline == self.ly_compare as usize {
                    self.status.coincidence = true;         
                    
                    if self.status.coincidence_interrupt_enable {
                        self.stat_interrupt = true;
                    }
                } else {
                    self.status.coincidence = false;
                }
            },

            PpuMode::VBLANK => {
                if self.mode_clocks >= PPU_VBLANK_CLOCKS {
                    self.mode_clocks -= PPU_VBLANK_CLOCKS;
                    self.scanline += 1;

                    if self.scanline == PPU_VBLANK_END {
                        self.mode = PpuMode::OAM;
                        self.scanline = 0;

                        if self.status.oam_interrupt_enable {
                            self.stat_interrupt = true;
                        }
                    }
                }

                if self.scanline == self.ly_compare as usize {
                    self.status.coincidence = true;

                    if self.status.coincidence_interrupt_enable {
                        self.stat_interrupt = true;
                    }
                } else {
                    self.status.coincidence = false;
                }
            },
        };
    }

    fn get_scrolled_y(&self) -> usize {
        (self.scanline + self.scroll_y as usize) % 256
    }

    fn get_scrolled_x(&self, x: usize) -> usize {
        (x + self.scroll_x as usize) % 256
    }

    fn get_bg_tile(&mut self, index: usize) -> u8 {
        let mut tile_base = match self.control.background_tile_address {
            true => 0x9c00,
            false => 0x9800,
        };

        tile_base += (self.get_scrolled_y() / 8) * 32;

        let tile_address = (tile_base + index) as u16;

        self.vram_read(tile_address)
    }

    fn get_bg_pattern(&mut self, tile: u8, row: usize, order: usize) -> u8 {
        let pattern_base;
        let pattern_tile;

        if self.control.background_pattern_address {
            pattern_base = 0x8000;
            pattern_tile = tile;
        } else {
            pattern_base = 0x8800;
            pattern_tile = (tile as i8 as isize).wrapping_add(0x80) as u8;
        }

        let pattern_address = pattern_base + (pattern_tile as usize * 16) + (row * 2) + order;

        self.vram_read(pattern_address as u16)
    }

    fn get_sprite_pattern(&mut self, tile: u8, row: usize, order: usize) -> u8 {
        let pattern_address = 0x8000 + (tile as usize * 16) + (row * 2) + order;
        self.vram_read(pattern_address as u16)
    }

    fn sprite_inrange(&mut self, y: isize) -> bool {
        y <= self.scanline as isize && self.scanline as isize <= (y + 7)
    }

    fn render_scanline(&mut self) {
        if self.control.background_enable {
            let y = self.get_scrolled_y();

            for i in 0..160 {
                let x = self.get_scrolled_x(i);

                let tile_number = self.get_bg_tile(x / 8);

                let tile_col = x % 8;
                let tile_row = y % 8;

                let tile_lo = self.get_bg_pattern(tile_number, tile_row, 0);
                let tile_hi = self.get_bg_pattern(tile_number, tile_row, 1);

                let framebuffer_address = self.scanline * 160 + i;

                let pixel_shade_lo = ((tile_lo << tile_col) & 0x80) >> 7;
                let pixel_shade_hi = ((tile_hi << tile_col) & 0x80) >> 6;
                let pixel_shade = pixel_shade_hi | pixel_shade_lo;

                self.framebuffer[framebuffer_address] = match pixel_shade {
                    0b00 => self.background_palette.colour0,
                    0b01 => self.background_palette.colour1,
                    0b10 => self.background_palette.colour2,
                    0b11 => self.background_palette.colour3,
                    _ => unreachable!(),
                };
            }
        }

        //if self.control.background_enable {
        //    for i in 0..20 {
        //        let x = self.get_scrolled_x(i * 8);
//
        //        let tile_number = self.get_bg_tile(x / 8);
        //        let tile_row = self.get_scrolled_y() % 8;
//
        //        let tile_lo = self.get_bg_pattern(tile_number, tile_row, 0);
        //        let tile_hi = self.get_bg_pattern(tile_number, tile_row, 1);
//
        //        for j in 0..8 {
        //            let framebuffer_address = (self.scanline * 160) + (i * 8) + j;
//
        //            let pixel_shade_hi = ((tile_hi << j) & 0x80) >> 6;
        //            let pixel_shade_lo = ((tile_lo << j) & 0x80) >> 7;
        //            let pixel_shade = pixel_shade_hi | pixel_shade_lo;
//
        //            self.framebuffer[framebuffer_address] = match pixel_shade {
        //                0b00 => self.background_palette.colour0,
        //                0b01 => self.background_palette.colour1,
        //                0b10 => self.background_palette.colour2,
        //                0b11 => self.background_palette.colour3,
        //                _ => unreachable!()
        //            };
        //        }
        //    }
        //}

        if self.control.sprite_enable {
            for i in 0..40 {
                let y = self.sprite_oam[i * 4] as isize - 16;
                let x = self.sprite_oam[(i * 4) + 1] as isize - 8;
                let sprite_tile = self.sprite_oam[(i * 4) + 2];
                let options = self.sprite_oam[(i * 4) + 3];

                if self.sprite_inrange(y) && y >= 0 && y < 144 {
                    let palette = match (options & 0x10) != 0 {
                        false => self.sprite_palette_0,
                        true => self.sprite_palette_1,
                    };

                    let sprite_row = self.scanline - y as usize;

                    let sprite_lo = self.get_sprite_pattern(sprite_tile, sprite_row, 0);
                    let sprite_hi = self.get_sprite_pattern(sprite_tile, sprite_row, 1);

                    for j in 0..8 {
                        if (x < 0) || (x as usize + j >= 160) {
                            continue;
                        }

                        let framebuffer_address = (self.scanline * 160) + x as usize + j;

                        let x_shift = match (options & 0x20) != 0 {
                            true => 7 - j,
                            false => j,
                        };

                        let sprite_shade_hi = ((sprite_hi << x_shift) & 0x80) >> 6;
                        let sprite_shade_lo = ((sprite_lo << x_shift) & 0x80) >> 7;

                        let sprite_shade = sprite_shade_hi | sprite_shade_lo;

                        let pixel_shade = match sprite_shade {
                            0b00 => palette.colour0,
                            0b01 => palette.colour1,
                            0b10 => palette.colour2,
                            0b11 => palette.colour3,
                            _ => unreachable!()
                        };

                        if sprite_shade != 0 {
                            if (options & 0x80) == 0 || self.framebuffer[framebuffer_address] == self.background_palette.colour0 {
                                self.framebuffer[framebuffer_address] = pixel_shade;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    pub fn get_vblank_status(&mut self) -> bool {
        if self.vblank {
            self.vblank = false;
            return true;
        }

        false
    }

    pub fn get_lcdc_status(&mut self) -> bool {
        if self.stat_interrupt {
            self.stat_interrupt = false;
            return true;
        }

        false
    }

    pub fn controller_read(&self) -> u8 {
        self.controller.read()
    }

    pub fn controller_write(&mut self, value: u8) {
        self.controller.write(value);
    }

    pub fn stat_read(&self) -> u8 {
        self.status.read(self.mode)
    }

    pub fn stat_write(&mut self, value: u8) {
        self.status.write(value);
    }

    pub fn lcdc_read(&self) -> u8 {
        self.control.read()
    }

    pub fn lcdc_write(&mut self, value: u8) {
        self.control.write(value);
    }

    pub fn scy_read(&self) -> u8 {
        self.scroll_y
    }

    pub fn scy_write(&mut self, value: u8) {
        self.scroll_y = value;
    }

    pub fn scx_read(&self) -> u8 {
        self.scroll_x
    }

    pub fn scx_write(&mut self, value: u8) {
        self.scroll_x = value;
    }

    pub fn ly_read(&self) -> u8 {
        self.scanline as u8
    }

    pub fn lyc_read(&self) -> u8 {
        self.ly_compare
    }

    pub fn lyc_write(&mut self, value: u8) {
        self.ly_compare = value;
    }

    pub fn bgp_read(&self) -> u8 {
        self.background_palette.read()
    }

    pub fn obp1_read(&self) -> u8 {
        self.sprite_palette_0.read()
    }

    pub fn obp2_read(&self) -> u8 {
        self.sprite_palette_1.read()
    }

    pub fn bgp_write(&mut self, value: u8) {
        self.background_palette.write(value);
    }

    pub fn obp1_write(&mut self, value: u8) {
        self.sprite_palette_0.write(value);
    }

    pub fn obp2_write(&mut self, value: u8) {
        self.sprite_palette_1.write(value);
    }
}

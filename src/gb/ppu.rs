use super::window;

pub const PPU_DISPLAY_WIDTH: usize = 160;
pub const PPU_DISPLAY_HEIGHT: usize = 144;

pub const PPU_OAM_CLOCKS: usize = 80;
pub const PPU_VRAM_CLOCKS: usize = 172;
pub const PPU_HBLANK_CLOCKS: usize = 204;
pub const PPU_VBLANK_CLOCKS: usize = 456;

pub const PPU_VBLANK_START: usize = 144;
pub const PPU_VBLANK_END: usize = 154;

enum PpuMode {
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
    //...
    //...
    sprite_size: PpuSpriteSize,
    sprite_enable: bool,
    background_enable: bool,
}

impl PpuControl {
    pub fn new() -> PpuControl {
        PpuControl {
            lcd_enable: false,
            window_enable: false,
            sprite_size: PpuSpriteSize::NORMAL,
            sprite_enable: false,
            background_enable: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.lcd_enable as u8)    << 7 |
        (self.window_enable as u8) << 5 |
        (self.sprite_size as u8)   << 2 |
        (self.sprite_enable as u8) << 1 |
        self.background_enable as u8
    }

    pub fn write(&mut self, value: u8) {
        self.lcd_enable = (value & 0x80) != 0;
        self.window_enable = (value & 0x20) != 0;

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
}

#[derive(Clone, Copy)]
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
}

pub struct Ppu {
    window: window::SdlContext,

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
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            window: window::SdlContext::new(PPU_DISPLAY_WIDTH, PPU_DISPLAY_HEIGHT, "rgb"),

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

    pub fn tick(&mut self) {
        self.mode_clocks += 4;

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

                    if self.control.lcd_enable && self.control.background_enable {
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
                        self.window.render(&self.framebuffer);
                        self.window.sleep_frame();
                    } else {
                        self.mode = PpuMode::OAM;
                    }
                }
            },

            PpuMode::VBLANK => {
                if self.mode_clocks >= PPU_VBLANK_CLOCKS {
                    self.mode_clocks -= PPU_VBLANK_CLOCKS;
                    self.scanline += 1;

                    if self.scanline == PPU_VBLANK_END {
                        self.mode = PpuMode::OAM;
                        self.scanline = 0;
                    }
                }
            },
        };
    }

    fn render_scanline(&mut self) {
        let scrolled_scanline = (self.scanline + self.scroll_y as usize) % 256;

        let background_map_address = (scrolled_scanline / 8) * 32;

        for i in 0..20 {
            let tile_number = self.background_ram[background_map_address + i];
            let tile_row = scrolled_scanline % 8;

            let tile_address = (tile_number as usize * 16) + (tile_row * 2);

            let tile_lo = self.tile_ram[tile_address];
            let tile_hi = self.tile_ram[tile_address + 1];

            for j in 0..8 {
                let framebuffer_address = (self.scanline * 160) + (i * 8) + j;

                let pixel_shade_hi = ((tile_hi << j) & 0x80) >> 6;
                let pixel_shade_lo = ((tile_lo << j) & 0x80) >> 7;
                let pixel_shade = pixel_shade_hi | pixel_shade_lo;

                self.framebuffer[framebuffer_address] = match pixel_shade {
                    0b00 => self.background_palette.colour0,
                    0b01 => self.background_palette.colour1,
                    0b10 => self.background_palette.colour2,
                    0b11 => self.background_palette.colour3,
                    _ => unreachable!()
                };
            }
        }
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

    pub fn ly_read(&self) -> u8 {
        self.scanline as u8
    }

    pub fn bgp_read(&self) -> u8 {
        ((self.background_palette.colour3 as u8) << 6) |
        ((self.background_palette.colour2 as u8) << 4) |
        ((self.background_palette.colour1 as u8) << 2) |
          self.background_palette.colour0 as u8
    }

    pub fn bgp_write(&mut self, value: u8) {
        self.background_palette.colour3 = PpuShade::from_u8((value >> 6) & 0x03);
        self.background_palette.colour2 = PpuShade::from_u8((value >> 4) & 0x03);
        self.background_palette.colour1 = PpuShade::from_u8((value >> 2) & 0x03);
        self.background_palette.colour0 = PpuShade::from_u8(value & 0x03);
    }
}

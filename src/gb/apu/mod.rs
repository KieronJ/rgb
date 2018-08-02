mod square;

use super::audio_system::AudioSystem;

use self::square::ApuSquare;

pub struct Apu {
    audio_system: AudioSystem,

    master_enable: bool,
    master_timer: usize,

    l_volume: usize,
    r_volume: usize,

    l_enable: [bool; 4],
    r_enable: [bool; 4],

    square1: ApuSquare,
}

impl Apu {
    pub fn new(audio_system: AudioSystem) -> Apu {
        Apu {
            audio_system: audio_system,

            master_enable: false,
            master_timer: 95,

            l_volume: 0,
            r_volume: 0,

            l_enable: [false; 4],
            r_enable: [false; 4],

            square1: ApuSquare::new(),
        }
    }

    pub fn tick(&mut self) {
        let square1 = self.square1.tick();
        let output = square1 as f32;

        self.master_timer -= 1;

        if self.master_timer == 0 {
            self.master_timer = 95;

            self.audio_system.add_samples(&[output]);
        }
    }

    pub fn nr11_write(&mut self, value: u8) {
        self.square1.write_1(value);
    }

    pub fn nr12_write(&mut self, value: u8) {
        self.square1.write_2(value);
    }

    pub fn nr13_write(&mut self, value: u8) {
        self.square1.write_3(value);
    }

    pub fn nr14_write(&mut self, value: u8) {
        self.square1.write_4(value);
    }

    pub fn nr50_write(&mut self, value: u8) {
        self.l_volume = ((value & 0x70) >> 4) as usize;
        self.r_volume = (value & 0x7) as usize;
    }

    pub fn nr51_write(&mut self, value: u8) {
        self.l_enable[3] = (value & 0x80) != 0;
        self.l_enable[2] = (value & 0x40) != 0;
        self.l_enable[1] = (value & 0x20) != 0;
        self.l_enable[0] = (value & 0x10) != 0;

        self.r_enable[3] = (value & 0x08) != 0;
        self.r_enable[2] = (value & 0x04) != 0;
        self.r_enable[1] = (value & 0x02) != 0;
        self.r_enable[0] = (value & 0x01) != 0;
    }

    pub fn nr52_write(&mut self, value: u8) {
        self.master_enable = (value & 0x80) != 0;
    }
}
mod square;

use super::audio_system::AudioSystem;

use self::square::ApuSquare;

pub struct Apu {
    audio_system: AudioSystem,

    master_enable: bool,
    master_timer: usize,

    l_volume: f32,
    r_volume: f32,

    l_enable: [bool; 4],
    r_enable: [bool; 4],

    sequencer_step: usize,
    sequencer_timer: usize,

    square1: ApuSquare,
    square2: ApuSquare,
}

impl Apu {
    pub fn new(audio_system: AudioSystem) -> Apu {
        Apu {
            audio_system: audio_system,

            master_enable: false,
            master_timer: 95,

            l_volume: 0.0,
            r_volume: 0.0,

            l_enable: [false; 4],
            r_enable: [false; 4],

            sequencer_step: 0,
            sequencer_timer: 8192,

            square1: ApuSquare::new(),
            square2: ApuSquare::new(),
        }
    }

    pub fn tick(&mut self) {
        if !self.master_enable {
            return;
        }

        self.sequencer_timer -= 1;

        if self.sequencer_timer == 0 {
            self.sequencer_timer = 8192;

            //if (self.sequencer_step & 0x1) == 0 {
            //    self.square1.tick_length();
            //    self.square2.tick_length();
            //    self.wave.tick_length();
            //    self.noise.tick_length();
            //}

            if self.sequencer_step == 0x7 {
                self.square1.tick_envelope();
                self.square2.tick_envelope();
                //self.noise.tick_envelope();
            }

            //if (self.sequencer_step & 0x3) == 0x2 {
            //    self.square1.tick_sweep();
            //}

            self.sequencer_step = (self.sequencer_step + 1) & 0x7;
        }

        let square1 = ((self.square1.tick() as f32) / 7.5) - 1.0;
        let square2 = ((self.square2.tick() as f32) / 7.5) - 1.0;

        let mut l_output = 0.0;
        let mut r_output = 0.0;

        if self.l_enable[0] {
            l_output += square1;
        } 

        if self.l_enable[1] {
            l_output += square2;
        } 

        if self.r_enable[0] {
            r_output += square1;
        }

        if self.r_enable[1] {
            r_output += square2;
        }

        l_output *= self.l_volume + 1.0;
        r_output *= self.r_volume + 1.0;

        self.master_timer -= 1;

        if self.master_timer == 0 {
            self.master_timer = 95;

            self.audio_system.add_samples(&[l_output, r_output]);
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

    pub fn nr21_write(&mut self, value: u8) {
        self.square2.write_1(value);
    }

    pub fn nr22_write(&mut self, value: u8) {
        self.square2.write_2(value);
    }

    pub fn nr23_write(&mut self, value: u8) {
        self.square2.write_3(value);
    }

    pub fn nr24_write(&mut self, value: u8) {
        self.square2.write_4(value);
    }

    pub fn nr50_write(&mut self, value: u8) {
        let l = (value & 0x70) >> 4;
        let r = value & 0x7;

        self.l_volume = (l as f32) / 7.0;
        self.r_volume = (r as f32) / 7.0;
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
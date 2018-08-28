mod noise;
mod square;
mod wave;

use super::audio_system::AudioSystem;

use self::noise::ApuNoise;
use self::square::ApuSquare;
use self::wave::ApuWave;

pub struct Apu {
    audio_system: AudioSystem,

    enable: bool,

    rs_period: usize,

    fs_period: usize,
    fs_step: usize,

    l_volume: usize,
    r_volume: usize,

    l_vin: bool,
    r_vin: bool,

    l_enable: [bool; 4],
    r_enable: [bool; 4],

    square1: ApuSquare,
    square2: ApuSquare,
    noise: ApuNoise,
    wave: ApuWave,
}

impl Apu {
    pub fn new(audio_system: AudioSystem) -> Apu {
        Apu {
            audio_system: audio_system,

            enable: false,

            rs_period: 0,

            fs_period: 0,
            fs_step: 0,

            l_volume: 0,
            r_volume: 0,

            l_vin: false,
            r_vin: false,

            l_enable: [false; 4],
            r_enable: [false; 4],

            square1: ApuSquare::new(),
            square2: ApuSquare::new(),
            wave: ApuWave::new(),
            noise: ApuNoise::new(),
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        if !self.enable {
            return;
        }

        for _ in 0..cycles {
            self.tick_frame_sequencer();

            let sq1 = self.square1.tick() as i16;
            let sq2 = self.square2.tick() as i16;
            let wave = self.wave.tick() as i16;
            let noise = self.noise.tick() as i16;

            let mut l_out = 0;
            if self.l_enable[0] { l_out += sq1; }
            if self.l_enable[1] { l_out += sq2; }
            if self.l_enable[2] { l_out += wave; }
            if self.l_enable[3] { l_out += noise; }
            l_out = (l_out * 512) - 16384;

            match self.l_volume {
                0 => l_out >>= 3,
                1 => l_out >>= 2,
                2 => l_out = (l_out >> 2) + (l_out >> 3),
                3 => l_out >>= 1,
                4 => l_out = (l_out >> 1) + (l_out >> 3),
                5 => l_out -= l_out >> 2,
                6 => l_out -= l_out >> 3,
                7 => (),
                _ => unreachable!(),
            }

            let mut r_out = 0;
            if self.r_enable[0] { r_out += sq1; }
            if self.r_enable[1] { r_out += sq2; }
            if self.r_enable[2] { r_out += wave; }
            if self.r_enable[3] { r_out += noise; }
            r_out = (r_out * 512) - 16384;

            match self.r_volume {
                0 => r_out >>= 3,
                1 => r_out >>= 2,
                2 => r_out = (r_out >> 2) + (r_out >> 3),
                3 => r_out >>= 1,
                4 => r_out = (r_out >> 1) + (r_out >> 3),
                5 => r_out -= r_out >> 2,
                6 => r_out -= r_out >> 3,
                7 => (),
                _ => unreachable!(),
            }

            if self.rs_period == 0 {
                self.rs_period = 95;

                self.audio_system.add_samples(&[l_out, r_out]);
            } else {
                self.rs_period -= 1;
            }
        }
    }

    fn tick_frame_sequencer(&mut self) {
        if self.fs_period == 0 {
            self.fs_period = 8192;
            self.fs_step = (self.fs_step + 1) & 0x7;

            if (self.fs_step & 1) == 0 {
                self.square1.tick_length();
                self.square2.tick_length();
                self.wave.tick_length();
                self.noise.tick_length();
            }

            if (self.fs_step & 3) == 2 {
                self.square1.tick_sweep();
            }

            if self.fs_step == 7 {
                self.square1.tick_envelope();
                self.square2.tick_envelope();
                self.noise.tick_envelope();
            }

            return;
        }

        self.fs_period -= 1;
    }

    pub fn nr10_read(&self) -> u8 {
        self.square1.read_0()
    }

    pub fn nr10_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square1.write_0(value);
    }

    pub fn nr11_read(&self) -> u8 {
        self.square1.read_1()
    }

    pub fn nr11_write(&mut self, value: u8) {
        if self.enable {
            self.square1.write_duty(value >> 6);
        }

        self.square1.write_length(value);
    }

    pub fn nr12_read(&self) -> u8 {
        self.square1.read_2()
    }

    pub fn nr12_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square1.write_2(value);
    }

    pub fn nr13_read(&self) -> u8 {
        self.square1.read_3()
    }

    pub fn nr13_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square1.write_3(value);
    }

    pub fn nr14_read(&self) -> u8 {
        self.square1.read_4()
    }

    pub fn nr14_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square1.write_4(value);
    }

    pub fn nr20_read(&mut self) -> u8 {
        0xff
    } 

    pub fn nr20_write(&mut self, _: u8) {
        return;
    }

    pub fn nr21_read(&mut self) -> u8 {
        self.square2.read_1()
    } 

    pub fn nr21_write(&mut self, value: u8) {
        if self.enable {
            self.square1.write_duty(value >> 6);
        }

        self.square1.write_length(value);
    }

    pub fn nr22_read(&mut self) -> u8 {
        self.square2.read_2()
    } 

    pub fn nr22_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square2.write_2(value);
    }

    pub fn nr23_read(&mut self) -> u8 {
        self.square2.read_3()
    }

    pub fn nr23_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square2.write_3(value);
    }

    pub fn nr24_read(&mut self) -> u8 {
        self.square2.read_4()
    }

    pub fn nr24_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.square2.write_4(value);
    }

    pub fn nr30_read(&mut self) -> u8 {
        self.wave.read_0()
    } 

    pub fn nr30_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.wave.write_0(value);
    }

    pub fn nr31_read(&mut self) -> u8 {
        self.wave.read_1()
    } 

    pub fn nr31_write(&mut self, value: u8) {
        self.wave.write_1(value);
    }

    pub fn nr32_read(&mut self) -> u8 {
        self.wave.read_2()
    } 

    pub fn nr32_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.wave.write_2(value);
    }

    pub fn nr33_read(&mut self) -> u8 {
        self.wave.read_3()
    }

    pub fn nr33_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.wave.write_3(value);
    }

    pub fn nr34_read(&mut self) -> u8 {
        self.wave.read_4()
    }

    pub fn nr34_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.wave.write_4(value);
    }

    pub fn nr40_read(&mut self) -> u8 {
        0xff
    } 

    pub fn nr40_write(&mut self, _: u8) {
        return;
    }

    pub fn nr41_read(&mut self) -> u8 {
        self.noise.read_1()
    } 

    pub fn nr41_write(&mut self, value: u8) {
        self.noise.write_1(value);
    }

    pub fn nr42_read(&mut self) -> u8 {
        self.noise.read_2()
    } 

    pub fn nr42_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.noise.write_2(value);
    }

    pub fn nr43_read(&mut self) -> u8 {
        self.noise.read_3()
    }

    pub fn nr43_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.noise.write_3(value);
    }

    pub fn nr44_read(&mut self) -> u8 {
        self.noise.read_4()
    }

    pub fn nr44_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.noise.write_4(value);
    }

    pub fn nr50_read(&mut self) -> u8 {
        let mut value = 0;

        value |= (self.l_vin as u8) << 7;
        value |= (self.l_volume as u8) << 4;
        value |= (self.r_vin as u8) << 3;
        value |= self.r_volume as u8;

        value
    }

    pub fn nr50_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.l_vin = (value & 0x80) != 0;
        self.l_volume = ((value & 0x70) >> 4) as usize;
        self.r_vin = (value & 0x8) != 0;
        self.r_volume = (value & 0x7) as usize;
    }

    pub fn nr51_read(&self) -> u8 {
        let mut value = 0;

        value |= (self.l_enable[3] as u8) << 7;
        value |= (self.l_enable[2] as u8) << 6;
        value |= (self.l_enable[1] as u8) << 5;
        value |= (self.l_enable[0] as u8) << 4;
        value |= (self.r_enable[3] as u8) << 3;
        value |= (self.r_enable[2] as u8) << 2;
        value |= (self.r_enable[1] as u8) << 1;
        value |= self.r_enable[0] as u8;

        value
    }

    pub fn nr51_write(&mut self, value: u8) {
        if !self.enable {
            return;
        }

        self.l_enable[3] = (value & 0x80) != 0;
        self.l_enable[2] = (value & 0x40) != 0;
        self.l_enable[1] = (value & 0x20) != 0;
        self.l_enable[0] = (value & 0x10) != 0;

        self.r_enable[3] = (value & 0x08) != 0;
        self.r_enable[2] = (value & 0x04) != 0;
        self.r_enable[1] = (value & 0x02) != 0;
        self.r_enable[0] = (value & 0x01) != 0;
    }

    pub fn nr52_read(&self) -> u8 {
        let mut value = 0x70;

        value |= (self.enable as u8) << 7;
        value |= (self.noise.enabled() as u8) << 3;
        value |= (self.wave.enabled() as u8) << 2;
        value |= (self.square2.enabled() as u8) << 1;
        value |= self.square1.enabled() as u8;

        value
    }

    pub fn nr52_write(&mut self, value: u8) {
        self.enable = (value & 0x80) != 0;

        if self.enable {
            self.audio_system.resume();
        } else {
            self.audio_system.pause();
        }

        if !self.enable {
            self.rs_period = 0;

            self.fs_period = 0;
            self.fs_step = 0;

            self.l_volume = 0;
            self.r_volume = 0;

            self.l_vin = false;
            self.r_vin = false;

            self.l_enable = [false, false, false, false];
            self.r_enable = [false, false, false, false];

            self.square1.reset();
            self.square2.reset();
            self.wave.reset();
            self.noise.reset();
        }
    }

    pub fn read_wavetable(&self, address: u16) -> u8 {
        let index = (address as usize) - 0xff30;

        self.wave.read_wavetable(index)
    }

    pub fn write_wavetable(&mut self, address: u16, value: u8) {
        let index = (address as usize) - 0xff30;

        self.wave.write_wavetable(index, value);
    }
}
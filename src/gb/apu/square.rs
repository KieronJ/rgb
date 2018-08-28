const DUTY: [[usize; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct ApuSquare {
    enabled: bool,

    period: usize,

    sweep_frequency: usize,
    sweep_period: usize,
    sweep_negate: bool,
    sweep_shift: usize,
    sweep_shadow: usize,

    duty: usize,
    phase: usize,

    length: usize,

    envelope_volume: usize,
    envelope_negate: bool,
    envelope_frequency: usize,
    envelope_period: usize,

    frequency: usize,

    counter: bool,

    volume: usize,
}

impl ApuSquare {
    pub fn new() -> ApuSquare {
        ApuSquare {
            enabled: false,

            period: 0,

            sweep_frequency: 0,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_shadow: 0,

            duty: 0,
            phase: 0,

            length: 0,
            
            envelope_volume: 0,
            envelope_negate: false,
            envelope_frequency: 0,
            envelope_period: 0,

            frequency: 0,

            counter: false,

            volume: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;

        self.period = 0;

        self.sweep_frequency = 0;
        self.sweep_period = 0;
        self.sweep_negate = false;
        self.sweep_shift = 0;
        self.sweep_shadow = 0;

        self.duty = 0;
        self.phase = 0;

        self.envelope_volume = 0;
        self.envelope_negate = false;
        self.envelope_frequency = 0;
        self.envelope_period = 0;

        self.frequency = 0;

        self.counter = false;

        self.volume = 0;
    }

    pub fn tick(&mut self) -> usize {
        if self.period == 0 {
            self.period = (2048 - self.frequency) << 2;
            self.phase = (self.phase + 1) & 0x7;
        } else {
            self.period -= 1;
        }

        if !self.enabled {
            return 0;
        }

        self.volume * DUTY[self.duty][self.phase]
    }

    pub fn tick_length(&mut self) {
        if self.counter && self.length != 0 {
            self.length -= 1;

            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn tick_envelope(&mut self) {
        if self.enabled && self.envelope_frequency != 0 {
            if self.envelope_period == 0 {
                self.envelope_period = self.envelope_frequency;

                if !self.envelope_negate && self.volume > 0 {
                    self.volume -= 1;
                }

                if self.envelope_negate && self.volume < 15 {
                    self.volume += 1;
                }

            } else {
                self.envelope_period -= 1;
            }
        }
    }

    pub fn tick_sweep(&mut self) {
        if self.enabled && self.sweep_frequency != 0 {
            if self.sweep_period == 0 {
                self.sweep_period = self.sweep_frequency;

                let additive = self.sweep_shadow >> self.sweep_shift;
                let mut new_freq = self.sweep_shadow;

                if self.sweep_negate {
                    new_freq -= additive;
                } else {
                    new_freq += additive;
                }

                if new_freq > 2047 {
                    self.enabled = false;
                    return;
                } else if self.sweep_shift != 0 {
                    self.sweep_shadow = new_freq;
                    self.frequency = new_freq;
                }

                let additive = self.sweep_shadow >> self.sweep_shift;
                let mut new_freq = self.sweep_shadow;

                if self.sweep_negate {
                    new_freq -= additive;
                } else {
                    new_freq += additive;
                }

                if new_freq > 2047 {
                    self.enabled = false;
                }

            } else {
                self.sweep_period -= 1;
            }
        }
    }

    pub fn read_0(&self) -> u8 {
        let mut value = 0x80;

        value |= (self.sweep_frequency as u8) << 4;
        value |= (self.sweep_negate as u8) << 3;
        value |= self.sweep_shift as u8;

        value
    }

    pub fn write_0(&mut self, value: u8) {
        self.sweep_frequency = ((value & 0x70) >> 4) as usize;

        if self.sweep_frequency == 0 {
            self.sweep_frequency = 8;
        }

        self.sweep_negate = (value & 0x8) != 0;
        self.sweep_shift = (value & 0x7) as usize;
    }

    pub fn read_1(&self) -> u8 {
        let mut value = 0x3f;

        value |= (self.duty as u8) << 6;

        value
    }

    pub fn write_duty(&mut self, value: u8) {
        self.duty = (value & 0x3) as usize;
    }

    pub fn write_length(&mut self, value: u8) {
        self.length = 64 - (value & 0x3f) as usize;
    }

    pub fn read_2(&self) -> u8 {
        let mut value = 0;

        value |= (self.envelope_volume as u8) << 4;
        value |= (self.envelope_negate as u8) << 3;
        value |= self.envelope_frequency as u8;

        value
    }

    pub fn write_2(&mut self, value: u8) {
        self.envelope_volume = (value >> 4) as usize;
        self.envelope_negate = (value & 0x8) != 0;
        self.envelope_frequency = (value & 0x7) as usize;

        if self.envelope_volume == 0 && !self.envelope_negate {
            self.enabled = false;
        }
    }

    pub fn read_3(&self) -> u8 {
        0xff
    }

    pub fn write_3(&mut self, value: u8) {
        self.frequency &= 0x700;
        self.frequency |= value as usize;
    }

    pub fn read_4(&self) -> u8 {
        let mut value = 0xbf;

        value |= (self.counter as u8) << 6;

        value
    }

    pub fn write_4(&mut self, value: u8) {
        let trigger = (value & 0x80) != 0;

        self.counter = (value & 0x40) != 0;

        self.frequency &= 0xff;
        self.frequency |= ((value & 0x7) as usize) << 8;

        if trigger {
            self.enabled = (self.envelope_volume != 0) || self.envelope_negate;

            if self.length == 0 {
                self.length = 64;
            }

            self.envelope_period = self.envelope_frequency;

            self.volume = self.envelope_volume;

            self.sweep_shadow = self.frequency;

            self.period = (2048 - self.frequency) << 2;
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
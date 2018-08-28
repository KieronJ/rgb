pub struct ApuNoise {
    lfsr: usize,
    lfsr_timer: usize,

    length: usize,

    envelope_volume: usize,
    envelope_negate: bool,
    envelope_frequency: usize,
    envelope_period: usize,
    volume: usize,

    clock_shift: usize,
    lfsr_width: bool,
    divisor: usize,

    counter: bool,

    enabled: bool,
}

impl ApuNoise {
    pub fn new() -> ApuNoise {
        ApuNoise {
            lfsr: 0,
            lfsr_timer: 0,

            length: 0,

            envelope_volume: 0,
            envelope_negate: false,
            envelope_frequency: 0,
            envelope_period: 0,
            volume: 0,

            clock_shift: 0,
            lfsr_width: false,
            divisor: 0,

            counter: false,

            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        self.lfsr = 0;
        self.lfsr_timer = 0;

        self.envelope_volume = 0;
        self.envelope_negate = false;
        self.envelope_frequency = 0;
        self.envelope_period = 0;
        self.volume = 0;

        self.clock_shift = 0;
        self.lfsr_width = false;
        self.divisor = 0;

        self.counter = false;

        self.enabled = false;
    }

    pub fn tick(&mut self) -> usize {
        if self.lfsr_timer == 0 {
            let mut divisor = self.divisor << 4;

            if divisor == 0 {
                divisor = 8;
            }

            self.lfsr_timer = divisor << self.clock_shift;

            let bit = (self.lfsr ^ (self.lfsr >> 1)) & 0x1;
            self.lfsr = (self.lfsr >> 1) ^ (bit << 14);

            if self.lfsr_width {
                self.lfsr ^= bit << 6;
            }
        } else {
            self.lfsr_timer -= 1;
        }

        if !self.enabled {
            return 0;
        }

        self.volume * (1 - (self.lfsr & 1))
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

    pub fn read_1(&self) -> u8 {
        0xff
    }

    pub fn write_1(&mut self, value: u8) {
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
        let mut value = 0;

        value |= (self.clock_shift as u8) << 4;
        value |= (self.lfsr_width as u8) << 3;
        value |= self.divisor as u8;

        value
    }

    pub fn write_3(&mut self, value: u8) {
        self.clock_shift = (value >> 4) as usize;
        self.lfsr_width = (value & 0x8) != 0;
        self.divisor = (value & 0x7) as usize;

        let mut divisor = self.divisor << 4;

        if self.divisor == 0 {
            divisor = 8;
        }

        self.lfsr_timer = divisor << self.clock_shift;
    }

    pub fn read_4(&self) -> u8 {
        let mut value = 0xbf;

        value |= (self.counter as u8) << 6;

        value
    }

    pub fn write_4(&mut self, value: u8) {
        let trigger = (value & 0x80) != 0;

        self.counter = (value & 0x40) != 0;

        if trigger {
            self.enabled = (self.envelope_volume != 0) || self.envelope_negate;

            if self.length == 0 {
                self.length = 64;
            }

            self.envelope_period = self.envelope_frequency;

            self.volume = self.envelope_volume;

            self.lfsr = 0x7fff;
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
pub struct ApuWave {
    wavetable: Box<[u8]>,
    index: usize,
    sample: usize,

    dac_enable: bool,

    length: usize,

    volume: usize,

    frequency: usize,
    period: usize,

    counter: bool,

    enabled: bool,
}

impl ApuWave {
    pub fn new() -> ApuWave {
        ApuWave {
            wavetable: vec![0x84, 0x40, 0x43, 0xaa, 0x2d, 0x78, 0x92, 0x3c, 0x60, 0x59, 0x59, 0xb0, 0x34, 0xb8, 0x2e, 0xda].into_boxed_slice(),
            index: 0,
            sample: 0,

            dac_enable: false,

            length: 0,

            volume: 0,

            frequency: 0,
            period: 0,

            counter: false,

            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        self.dac_enable = false;

        self.volume = 0;

        self.frequency = 0;

        self.counter = false;

        self.enabled = false;
    }

    pub fn tick(&mut self) -> usize {
        if self.period == 0 {
            self.period = (2048 - self.frequency) << 1;
            self.index = (self.index + 1) & 0x1f;

            let sample_byte = self.wavetable[self.index >> 1];

            if (self.index & 1) != 0 {
                self.sample = (sample_byte & 0xf) as usize;
            } else {
                self.sample = (sample_byte >> 4) as usize;
            }

        } else {
            self.period -= 1;
        }

        if !self.enabled || !self.dac_enable {
            return 0;
        }

        match self.volume {
            0 => 0,
            1 => self.sample,
            2 => self.sample >> 1,
            3 => self.sample >> 2,
            _ => unreachable!(),
        }
    }

    pub fn tick_length(&mut self) {
        if self.counter && self.length != 0 {
            self.length -= 1;

            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn read_0(&self) -> u8 {
        let mut value = 0x7f;

        value |= (self.dac_enable as u8) << 7;

        value
    }

    pub fn write_0(&mut self, value: u8) {
        self.dac_enable = (value & 0x80) != 0;
    }

    pub fn read_1(&self) -> u8 {
        0xff
    }

    pub fn write_1(&mut self, value: u8) {
        self.length = 256 - value as usize;
    }

    pub fn read_2(&self) -> u8 {
        let mut value = 0x9f;

        value |= (self.volume as u8) << 5;

        value
    }

    pub fn write_2(&mut self, value: u8) {
        self.volume = ((value & 0x60) >> 5) as usize;
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
            self.enabled = self.dac_enable;

            if self.length == 0 {
                self.length = 256;
            }

            self.period = (2048 - self.frequency) << 1;

            self.index = 0;
        }
    }

    pub fn read_wavetable(&self, index: usize) -> u8 {
        self.wavetable[index]
    }

    pub fn write_wavetable(&mut self, index: usize, value: u8) {
        self.wavetable[index] = value;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
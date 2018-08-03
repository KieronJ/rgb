const DUTY: [[usize; 8]; 4] = [
    [1, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

pub struct ApuSquare {
    timer_counter: usize,

    duty: usize,
    duty_step: usize,

    envelope_timer: usize,
    envelope_initial_volume: usize,
    envelope_volume: usize,
    envelope_negate: bool,
    envelope_period: usize,

    length: usize,

    frequency: usize,

    counter: bool,
}

impl ApuSquare {
    pub fn new() -> ApuSquare {
        ApuSquare {
            timer_counter: 8192,

            duty: 0,
            duty_step: 0,

            envelope_timer: 0,
            envelope_initial_volume: 0,
            envelope_volume: 0,
            envelope_negate: false,
            envelope_period: 0,

            length: 0,

            frequency: 0,

            counter: false,
        }
    }

    pub fn tick(&mut self) -> usize {
        self.timer_counter -= 1;

        if self.timer_counter == 0 {
            self.timer_counter = (2048 - self.frequency) * 4;

            self.duty_step = (self.duty_step + 1) & 0x7;
        }

        DUTY[self.duty][self.duty_step] * self.envelope_volume
    }

    pub fn tick_envelope(&mut self) {
        if self.envelope_period != 0 {
            self.envelope_timer -= 1;

            if self.envelope_timer == 0 {
                self.envelope_timer = self.envelope_period;

                if !self.envelope_negate {
                    if self.envelope_volume < 0xf {
                        self.envelope_volume += 1;
                    }
                } else {
                    if self.envelope_volume > 0 {
                        self.envelope_volume -= 1;
                    }
                }
            }
        }
    }

    pub fn write_1(&mut self, value: u8) {
        self.duty = ((value & 0xc0) >> 6) as usize;
        self.length = (value & 0x3f) as usize;
    }

    pub fn write_2(&mut self, value: u8) {
        self.envelope_initial_volume = ((value & 0xf0) >> 4) as usize;
        self.envelope_negate = (value & 0x8) != 0;
        self.envelope_period = (value & 0x7) as usize;
    }

    pub fn write_3(&mut self, value: u8) {
        self.frequency &= 0x700;
        self.frequency |= value as usize;
    }

    pub fn write_4(&mut self, value: u8) {
        self.counter = (value & 0x40) != 0;

        self.frequency &= 0xff;
        self.frequency |= ((value & 0x7) as usize) << 8;

        if (value & 0x80) != 0 {
            self.timer_counter = (2048 - self.frequency) * 4;
            self.envelope_timer = self.envelope_period;
            self.envelope_volume = self.envelope_initial_volume;
        }
    }
}
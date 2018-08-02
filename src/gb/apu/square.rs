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

    length: usize,

    frequency: usize,

    restart: bool,
    counter: bool,
}

impl ApuSquare {
    pub fn new() -> ApuSquare {
        ApuSquare {
            timer_counter: 8192,

            duty: 0,
            duty_step: 0,

            length: 0,

            frequency: 0,

            restart: false,
            counter: false,
        }
    }

    pub fn tick(&mut self) -> usize {
        self.timer_counter -= 1;

        if self.timer_counter == 0 {
            self.timer_counter = (2048 - self.frequency) * 4;

            self.duty_step = (self.duty_step + 1) & 0x7;
        }

        DUTY[self.duty][self.duty_step]
    }

    pub fn write_1(&mut self, value: u8) {
        self.duty = ((value & 0xc0) >> 6) as usize;
        self.length = (value & 0x3f) as usize;

        println!("Duty: {}", self.duty);
        println!("Length: {}", self.length);
    }

    pub fn write_2(&mut self, value: u8) {
        println!("[APU] [WARN]: Envelope unimplemented");
    }

    pub fn write_3(&mut self, value: u8) {
        self.frequency &= 0x700;
        self.frequency |= value as usize;

        println!("Frequency (Post-LSB): {}", self.frequency);
    }

    pub fn write_4(&mut self, value: u8) {
        self.restart = (value & 0x80) != 0;
        self.counter = (value & 0x40) != 0;

        self.frequency &= 0xff;
        self.frequency |= ((value & 0x7) as usize) << 8;

        println!("Restart: {}", self.restart);
        println!("Counter: {}", self.counter);
        println!("Frequency (Post-MSB): {}", self.frequency);
    }
}
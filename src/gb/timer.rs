#[derive(Clone, Copy)]
enum InputClock {
    MODE0,
    MODE1,
    MODE2,
    MODE3,
}

pub struct Timer {
    divider: u16,

    counter: u8,
    counter_cycles: usize,

    modulo: u8,
    enable: bool,
    clock: InputClock,

    interrupt: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,

            counter: 0,
            counter_cycles: 1024,

            modulo: 0,
            enable: false,
            clock: InputClock::MODE0,

            interrupt: false,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.tick_divider();
            self.tick_counter();
        }
    }

    fn tick_divider(&mut self) {
        self.divider = self.divider.wrapping_add(1);
    }

    fn tick_counter(&mut self) {
        if !self.enable {
            return;
        }

        self.counter_cycles -= 1;

        if self.counter_cycles == 0 {
            self.counter_cycles = match self.clock {
                InputClock::MODE0 => 1024,
                InputClock::MODE1 => 16,
                InputClock::MODE2 => 64,
                InputClock::MODE3 => 256,
            };

            if self.counter != 0xff {
                self.counter += 1;
            } else {
                self.counter = self.modulo;
                self.interrupt = true;
            }
        }
    }

    pub fn div_read(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub fn div_write(&mut self, _: u8) {
        self.divider = 0;
    }

    pub fn tima_read(&self) -> u8 {
        self.counter
    }

    pub fn tima_write(&mut self, value: u8) {
        self.counter = value;
    }

    pub fn tma_read(&self) -> u8 {
        self.modulo
    }

    pub fn tma_write(&mut self, value: u8) {
        self.modulo = value;
    }

    pub fn tac_read(&self) -> u8 {
        let mut value = 0;

        if self.enable {
            value |= 0x04;
        }

        value |= match self.clock {
            InputClock::MODE0 => 0x00,
            InputClock::MODE1 => 0x01,
            InputClock::MODE2 => 0x02,
            InputClock::MODE3 => 0x03,
        };

        value
    }

    pub fn tac_write(&mut self, value: u8) {
        self.enable = (value & 0x04) != 0;

        self.clock = match value & 0x03 {
            0x00 => InputClock::MODE0,
            0x01 => InputClock::MODE1,
            0x02 => InputClock::MODE2,
            0x03 => InputClock::MODE3,
            _ => unreachable!(),
        };

        self.counter_cycles = match self.clock {
            InputClock::MODE0 => 1024,
            InputClock::MODE1 => 16,
            InputClock::MODE2 => 64,
            InputClock::MODE3 => 256,
        };
    }

    pub fn get_interrupt_status(&mut self) -> bool {
        if self.interrupt {
            self.interrupt = false;
            return true;
        }

        false
    }
}
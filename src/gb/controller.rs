use sdl2::keyboard::Keycode;

pub struct Controller {
    button_select: bool,
    direction_select: bool,

    up: bool,
    down: bool,
    left: bool,
    right: bool,
    start: bool,
    select: bool,
    b: bool,
    a: bool,

    interrupt: bool,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            button_select: false,
            direction_select: false,

            up: true,
            down: true,
            left: true,
            right: true,
            start: true,
            select: true,
            b: true,
            a: true,

            interrupt: false,
        }
    }

    pub fn read(&self) -> u8 {
        match (self.button_select, self.direction_select) {
            (true, true) => {
                0
            },
            (true, false) => {
                let mut state = 0x10;
                state |= (self.down as u8) << 3;
                state |= (self.up as u8) << 2;
                state |= (self.left as u8) << 1;
                state |= self.right as u8;
                state
            },
            (false, true) => {
                let mut state = 0x20;
                state |= (self.start as u8) << 3;
                state |= (self.select as u8) << 2;
                state |= (self.b as u8) << 1;
                state |= self.a as u8;
                state
            },
            (false, false) => {
                let mut state = 0x30;
                state |= ((self.down | self.start) as u8) << 3;
                state |= ((self.up |self.select) as u8) << 2;
                state |= ((self.left | self.b) as u8) << 1;
                state |= (self.right | self.a) as u8;
                state
            },
        }
    }

    pub fn write(&mut self, value: u8) {
        self.button_select = (value & 0x20) != 0;
        self.direction_select = (value & 0x10) != 0;
    }

    pub fn set(&mut self, keycode: Keycode, state: bool) {
        let mut valid = true;
        let mut prev = true;

        match keycode {
            Keycode::A => {prev = self.a; self.a = !state},
            Keycode::S =>  {prev = self.b; self.b = !state},
            Keycode::Z =>  {prev = self.start; self.start = !state},
            Keycode::X =>  {prev = self.select; self.select = !state},
            Keycode::Up =>  {prev = self.up; self.up = !state},
            Keycode::Down =>  {prev = self.down; self.down = !state},
            Keycode::Left =>  {prev = self.left; self.left = !state},
            Keycode::Right =>  {prev = self.right; self.right = !state},
            _ => valid = false,
        };

        if valid && (prev == true) && (state == false) {
            self.interrupt = true;
        }
    }

    pub fn get_interrupt_status(&mut self) -> bool {
        if self.interrupt {
            self.interrupt = false;
            return true;
        }

        false
    }
}
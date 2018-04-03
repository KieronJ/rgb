bitflags! {
    pub struct Flags: u8 {
        const ZERO      = 0b10000000;
        const SUBTRACT  = 0b01000000;
        const HALFCARRY = 0b00100000;
        const CARRY     = 0b00010000;
    }
}

#[derive(Clone, Copy)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: Flags,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            pc: 0,
            sp: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Flags::empty(),
            h: 0,
            l: 0,
        }
    }

    pub fn read8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn write8(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::A => self.a = value,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
        }
    }

    pub fn read16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => ((self.a as u16) << 8) | self.f.bits() as u16,
            Reg16::BC => ((self.b as u16) << 8) | self.c as u16,
            Reg16::DE => ((self.d as u16) << 8) | self.e as u16,
            Reg16::HL => ((self.h as u16) << 8) | self.l as u16,
            Reg16::SP => self.sp,
        }
    }

    pub fn write16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::AF => { self.a = (value >> 8) as u8; self.f = Flags::from_bits_truncate(value as u8) },
            Reg16::BC => { self.b = (value >> 8) as u8; self.c = value as u8; },
            Reg16::DE => { self.d = (value >> 8) as u8; self.e = value as u8; },
            Reg16::HL => { self.h = (value >> 8) as u8; self.l = value as u8; },
            Reg16::SP => { self.sp = value; },
        }
    }
}
mod registers;

use self::registers::{Registers, Flags, Reg8, Reg16};
use self::registers::Reg8::{A, B, C, D, E, H, L};
use self::registers::Reg16::{AF, BC, DE, HL, SP};

use super::bus;

pub struct Z80 {
    bus: bus::Bus,
    regs: Registers,
}

impl Z80 {
    pub fn new(bus: bus::Bus) -> Z80 {
        Z80 {
            bus: bus,
            regs: Registers::new(),
        }
    }

    pub fn reset(&mut self) {
        self.regs.pc = 0;
    }

    pub fn run(&mut self) {
        if self.regs.pc >= 0x100 {
            panic!("INFO: finished bootstrap");
        }

        let pc = self.imm();
        let opcode = self.bus.read(pc);

        //print!("0x{:04x}: ", pc);

        match opcode {
            0x04 => self.inc(B),
            0x05 => self.dec(B),
            0x06 => self.ld_imm(B),
            0x0c => self.inc(C),
            0x0d => self.dec(C),
            0x0e => self.ld_imm(C),
            0x11 => self.ld_imm16(DE),
            0x13 => self.inc16(DE),
            0x15 => self.dec(D),
            0x16 => self.ld_imm(D),
            0x17 => self.alu_rl(A),
            0x18 => self.jr(),
            0x1a => self.ld_a_nn(DE),
            0x1d => self.dec(E),
            0x1e => self.ld_imm(E),
            0x20 => self.jr_cond(Flags::ZERO, false),
            0x21 => self.ld_imm16(HL),
            0x22 => self.ld_hlpp_a(),
            0x23 => self.inc16(HL),
            0x24 => self.inc(H),
            0x28 => self.jr_cond(Flags::ZERO, true),
            0x2e => self.ld_imm(L),
            0x31 => self.ld_imm16(SP),
            0x32 => self.ld_hlmp_a(),
            0x3d => self.dec(A),
            0x3e => self.ld_imm(A),
            0x4f => self.ld_n_n(C, A),
            0x57 => self.ld_n_n(D, A),
            0x67 => self.ld_n_n(H, A),
            0x77 => self.ld_hlp(A),
            0x78 => self.ld_n_n(A, B),
            0x7b => self.ld_n_n(A, E),
            0x7c => self.ld_n_n(A, H),
            0x7d => self.ld_n_n(A, L),
            0x86 => self.add_hlp(),
            0x90 => self.sub(B),
            0xaf => self.xor(A),
            0xbe => self.cp_hlp(),
            0xc1 => self.pop_nn(BC),
            0xc5 => self.push_nn(BC),
            0xc9 => self.ret(),
            0xcb => self.cb_instr(),
            0xcd => self.call(),
            0xe0 => self.ldh_n_a(),
            0xe2 => self.ld_cp_a(),
            0xea => self.ld_imm16_a(),
            0xf0 => self.ldh_a_n(),
            0xfe => self.cp_imm(),
            _ => panic!("ERROR: unknown instruction 0x{:02x}", opcode)
        }

        //println!("AF:{:04x} BC:{:04x} DE:{:04x} HL:{:04x} SP:{:04x}", self.regs.read16(AF), self.regs.read16(BC), self.regs.read16(DE), self.regs.read16(HL), self.regs.read16(SP));
    }

    fn read8(&mut self, address: u16) -> u8 {
        self.bus.tick();
        self.bus.read(address)
    }

    fn read16(&mut self, address: u16) -> u16 {
        (self.bus.read(address) as u16) | ((self.bus.read(address + 1) as u16) << 8)
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.bus.tick();
        self.bus.write(address, value);
    }

    fn write16(&mut self, address: u16, value: u16) {
        self.bus.write(address, (value >> 8) as u8);
        self.bus.write(address + 1, value as u8);
    }

    pub fn imm(&mut self) -> u16 {
        self.regs.pc += 1;
        self.regs.pc - 1
    }

    pub fn imm16(&mut self) -> u16 {
        self.regs.pc += 2;
        self.regs.pc - 2
    }

    fn add_hlp(&mut self) {
        let address = self.regs.read16(HL);
        let value = self.read8(address);
        let a = self.regs.a;

        let result = a.wrapping_add(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) + (value & 0x0f) > 0x0f);
        self.regs.f.set(Flags::CARRY, (a as usize) + (value as usize) > 0xff);
    }

    fn sub(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);
        let a = self.regs.a;

        let result = a.wrapping_sub(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f));
        self.regs.f.set(Flags::CARRY, a < value);
    }

    fn cp_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);
        let a = self.regs.a;

        self.regs.f.set(Flags::ZERO, a == value);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f));
        self.regs.f.set(Flags::CARRY, a < value);
    }

    fn cp_hlp(&mut self) {
        let address = self.regs.read16(HL);
        let value = self.read8(address);

        let a = self.regs.a;

        self.regs.f.set(Flags::ZERO, a == value);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f));
        self.regs.f.set(Flags::CARRY, a < value);
    }

    fn pop_nn(&mut self, reg: Reg16) {
        let value = self.pop16();
        self.regs.write16(reg, value);
    }

    fn push_nn(&mut self, reg: Reg16) {
        let value = self.regs.read16(reg);
        self.push16(value);
    }

    fn ld_n_n(&mut self, dest: Reg8, src: Reg8) {
        let value = self.regs.read8(src);
        self.regs.write8(dest, value);
    }

    fn ret(&mut self) {
        let dest = self.pop16();
        self.regs.pc = dest;
    }

    fn inc(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);
        
        self.regs.write8(reg, value.wrapping_add(1));
        self.regs.f.set(Flags::ZERO, value == 0xff);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (value & 0x0f) == 0x0f);
    }

    fn inc16(&mut self, reg: Reg16) {
        let value = self.regs.read16(reg);
        
        self.regs.write16(reg, value.wrapping_add(1));
    }

    fn dec(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);
        
        self.regs.write8(reg, value.wrapping_sub(1));
        self.regs.f.set(Flags::ZERO, value == 0x01);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (value & 0x0f) == 0);
    }

    fn ld_imm(&mut self, reg: Reg8) {
        let pc = self.imm();
        let value = self.read8(pc);
        self.regs.write8(reg, value);
    }

    fn ld_a_nn(&mut self, reg: Reg16) {
        let address = self.regs.read16(reg);

        self.regs.a = self.read8(address);
    }

    fn ld_imm16_a(&mut self) {
        let pc = self.imm16();
        let address = self.read16(pc);
        let value = self.regs.a;

        self.write8(address, value);
    }

    fn ld_hlp(&mut self, reg: Reg8) {
        let hl = self.regs.read16(HL);
        let value = self.regs.read8(reg);

        self.write8(hl, value);
    }

    fn jr(&mut self,) {
        let pc = self.imm();
        let value = self.read8(pc) as i8;

        self.regs.pc = self.regs.pc.wrapping_add(value as u16);
    }

    fn jr_cond(&mut self, flag: Flags, state: bool) {
        let pc = self.imm();
        let value = self.read8(pc) as i8;

        if !state ^ self.regs.f.contains(flag) {
            self.regs.pc = self.regs.pc.wrapping_add(value as u16);
        }
    }

    fn ld_imm16(&mut self, reg: Reg16) {
        let pc = self.imm16();
        let value = self.read16(pc);
        self.regs.write16(reg, value);
    }

    fn ld_hlpp_a(&mut self) {
        let a = self.regs.a;
        let hl = self.regs.read16(HL);
        
        self.write8(hl, a);
        self.regs.write16(HL, hl.wrapping_add(1));
    }

    fn ld_hlmp_a(&mut self) {
        let a = self.regs.a;
        let hl = self.regs.read16(HL);
        
        self.write8(hl, a);
        self.regs.write16(HL, hl.wrapping_sub(1));
    }

    fn xor(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        self.regs.a ^= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
    }

    fn cb_instr(&mut self) {
        let pc = self.imm();
        let opcode = self.read8(pc);

        match opcode {
            0x11 => self.alu_rl(C),
            0x7c => self.cb_bit(7, H),
            _ => panic!("ERROR: unknown cb instruction 0x{:02x}", opcode)
        }
    }

    fn alu_rl(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);
        let carry = self.regs.f.contains(Flags::CARRY) as u8;

        let result = (value << 1) | carry;

        self.regs.write8(reg, result);
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x80) != 0);
    }

    fn cb_bit(&mut self, bit: usize, reg: Reg8) {
        let value = self.regs.read8(reg);

        assert!(bit <= 7);

        self.regs.f.set(Flags::ZERO, value & (1 << bit) == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, true);
    }

    fn ldh_n_a(&mut self) {
        let pc = self.imm();
        let address = 0xff00 + self.read8(pc) as u16;
        let a = self.regs.a;

        self.write8(address, a);
    }

    fn ldh_a_n(&mut self) {
        let pc = self.imm();
        let address = 0xff00 + self.read8(pc) as u16;

        self.regs.a = self.read8(address);
    }

    fn ld_cp_a(&mut self) {
        let address = 0xff00 + self.regs.c as u16;
        let value = self.regs.a;

        self.write8(address, value);
    }

    fn pop8(&mut self) -> u8 {
        self.regs.sp += 1;
        let sp = self.regs.sp;

        self.read8(sp)
    }

    fn pop16(&mut self) -> u16 {
        (self.pop8() as u16) | ((self.pop8() as u16) << 8)
    }

    fn push8(&mut self, value: u8) {
        let sp = self.regs.sp;
        self.regs.sp -= 1;

        self.write8(sp, value);
    }

    fn push16(&mut self, value: u16) {
        self.push8((value >> 8) as u8);
        self.push8(value as u8);
    }

    fn call(&mut self) {
        let pc = self.imm16();
        let dest = self.read16(pc);

        self.push16(pc + 2);
        self.regs.pc = dest;
    }
}
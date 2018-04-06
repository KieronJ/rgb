mod registers;

use self::registers::{Registers, Flags, Reg8, Reg16};
use self::registers::Reg8::{A, B, C, D, E, H, L};
use self::registers::Reg16::{AF, BC, DE, HL, SP};

use super::bus;

enum Cond {
    C, NC,
    Z, NZ
}

pub struct Z80 {
    bus: bus::Bus,
    regs: Registers,
    ime: bool,
}

impl Z80 {
    pub fn new(bus: bus::Bus) -> Z80 {
        Z80 {
            bus: bus,
            regs: Registers::new(),
            ime: false,
        }
    }

    pub fn reset(&mut self) {
        self.regs.pc = 0;
    }

    pub fn run(&mut self) {
        let pc = self.imm();
        let opcode = self.bus.read(pc);

        if pc == 0x100 {
            println!("INFO: finished bootstrap");
        }

        //if pc >= 0x100 {
        //    print!("0x{:04x}: ", pc);
        //}

        match opcode {
            0x00 => {}, // nop
            0x01 => self.ld_imm16(BC),
            0x03 => self.inc16(BC),
            0x04 => self.inc(B),
            0x05 => self.dec(B),
            0x06 => self.ld_imm(B),
            0x07 => self.rlca(),
            0x08 => self.ld_nn_sp(),
            0x09 => self.add_hl_nn(BC),
            0x0b => self.dec16(BC),
            0x0c => self.inc(C),
            0x0d => self.dec(C),
            0x0e => self.ld_imm(C),
            0x11 => self.ld_imm16(DE),
            0x12 => self.ld_nn_a(DE),
            0x13 => self.inc16(DE),
            0x14 => self.inc(D),
            0x15 => self.dec(D),
            0x16 => self.ld_imm(D),
            0x17 => self.rla(),
            0x18 => self.jr(),
            0x19 => self.add_hl_nn(DE),
            0x1a => self.ld_a_nn(DE),
            0x1b => self.dec16(DE),
            0x1c => self.inc(E),
            0x1d => self.dec(E),
            0x1e => self.ld_imm(E),
            0x1f => self.rra(),
            0x20 => self.jr_cond(Cond::NZ),
            0x21 => self.ld_imm16(HL),
            0x22 => self.ld_hlpp_a(),
            0x23 => self.inc16(HL),
            0x24 => self.inc(H),
            0x25 => self.dec(H),
            0x26 => self.ld_imm(H),
            0x28 => self.jr_cond(Cond::Z),
            0x29 => self.add_hl_nn(HL),
            0x2a => self.ld_a_hlpp(),
            0x2b => self.dec16(HL),
            0x2c => self.inc(L),
            0x2d => self.dec(L),
            0x2e => self.ld_imm(L),
            0x2f => self.cpl(),
            0x30 => self.jr_cond(Cond::NC),
            0x31 => self.ld_imm16(SP),
            0x32 => self.ld_hlmp_a(),
            0x33 => self.inc16(SP),
            0x35 => self.dec_hlp(),
            0x36 => self.ld_hlp_imm(),
            0x38 => self.jr_cond(Cond::C),
            0x39 => self.add_hl_nn(SP),
            0x3b => self.dec16(SP),
            0x3c => self.inc(A),
            0x3d => self.dec(A),
            0x3e => self.ld_imm(A),
            0x40 => self.ld_n_n(B, B),
            0x41 => self.ld_n_n(B, C),
            0x42 => self.ld_n_n(B, D),
            0x43 => self.ld_n_n(B, E),
            0x44 => self.ld_n_n(B, H),
            0x45 => self.ld_n_n(B, L),
            0x46 => self.ld_n_hlp(B),
            0x47 => self.ld_n_n(B, A),
            0x48 => self.ld_n_n(C, B),
            0x49 => self.ld_n_n(C, C),
            0x4a => self.ld_n_n(C, D),
            0x4b => self.ld_n_n(C, E),
            0x4c => self.ld_n_n(C, H),
            0x4d => self.ld_n_n(C, L),
            0x4e => self.ld_n_hlp(C),
            0x4f => self.ld_n_n(C, A),
            0x50 => self.ld_n_n(D, B),
            0x51 => self.ld_n_n(D, C),
            0x52 => self.ld_n_n(D, D),
            0x53 => self.ld_n_n(D, E),
            0x54 => self.ld_n_n(D, H),
            0x55 => self.ld_n_n(D, L),
            0x56 => self.ld_n_hlp(D),
            0x57 => self.ld_n_n(D, A),
            0x58 => self.ld_n_n(E, B),
            0x59 => self.ld_n_n(E, C),
            0x5a => self.ld_n_n(E, D),
            0x5b => self.ld_n_n(E, E),
            0x5c => self.ld_n_n(E, H),
            0x5d => self.ld_n_n(E, L),
            0x5e => self.ld_n_hlp(E),
            0x5f => self.ld_n_n(E, A),
            0x60 => self.ld_n_n(H, B),
            0x61 => self.ld_n_n(H, C),
            0x62 => self.ld_n_n(H, D),
            0x63 => self.ld_n_n(H, E),
            0x64 => self.ld_n_n(H, H),
            0x65 => self.ld_n_n(H, L),
            0x66 => self.ld_n_hlp(H),
            0x67 => self.ld_n_n(H, A),
            0x68 => self.ld_n_n(L, B),
            0x69 => self.ld_n_n(L, C),
            0x6a => self.ld_n_n(L, D),
            0x6b => self.ld_n_n(L, E),
            0x6c => self.ld_n_n(L, H),
            0x6d => self.ld_n_n(L, L),
            0x6e => self.ld_n_hlp(L),
            0x6f => self.ld_n_n(L, A),
            0x70 => self.ld_hlp_n(B),
            0x71 => self.ld_hlp_n(C),
            0x72 => self.ld_hlp_n(D),
            0x73 => self.ld_hlp_n(E),
            0x74 => self.ld_hlp_n(H),
            0x75 => self.ld_hlp_n(L),
            0x77 => self.ld_hlp_n(A),
            0x78 => self.ld_n_n(A, B),
            0x79 => self.ld_n_n(A, C),
            0x7a => self.ld_n_n(A, D),
            0x7b => self.ld_n_n(A, E),
            0x7c => self.ld_n_n(A, H),
            0x7d => self.ld_n_n(A, L),
            0x7e => self.ld_n_hlp(A),
            0x7f => self.ld_n_n(A, A),
            0x86 => self.add_hlp(),
            0x87 => self.add(A),
            0x90 => self.sub(B),
            0xa1 => self.and(C),
            0xa7 => self.and(A),
            0xa9 => self.xor(C),
            0xaa => self.xor(D),
            0xab => self.xor(E),
            0xac => self.xor(H),
            0xad => self.xor(L),
            0xae => self.xor_hlp(),
            0xaf => self.xor(A),
            0xb0 => self.or(B),
            0xb1 => self.or(C),
            0xb6 => self.or_hlp(),
            0xb7 => self.or(A),
            0xb9 => self.cp(C),
            0xbb => self.cp(E),
            0xbe => self.cp_hlp(),
            0xc0 => self.ret_cond(Cond::NZ),
            0xc1 => self.pop_nn(BC),
            0xc2 => self.jp_cond(Cond::NZ),
            0xc3 => self.jp(),
            0xc4 => self.call_cond(Cond::NZ),
            0xc5 => self.push_nn(BC),
            0xc6 => self.add_imm(),
            0xc7 => self.rst(0x00),
            0xc8 => self.ret_cond(Cond::Z),
            0xc9 => self.ret(),
            0xca => self.jp_cond(Cond::Z),
            0xcb => self.cb_instr(),
            0xcc => self.call_cond(Cond::Z),
            0xcd => self.call(),
            0xce => self.adc_imm(),
            0xcf => self.rst(0x08),
            0xd0 => self.ret_cond(Cond::NC),
            0xd1 => self.pop_nn(DE),
            0xd2 => self.jp_cond(Cond::NC),
            0xd4 => self.call_cond(Cond::NC),
            0xd5 => self.push_nn(DE),
            0xd6 => self.sub_imm(),
            0xd7 => self.rst(0x10),
            0xd8 => self.ret_cond(Cond::C),
            0xda => self.jp_cond(Cond::C),
            0xdc => self.call_cond(Cond::C),
            0xde => self.sbc_imm(),
            0xdf => self.rst(0x18),
            0xe0 => self.ldh_n_a(),
            0xe1 => self.pop_nn(HL),
            0xe2 => self.ld_cp_a(),
            0xe5 => self.push_nn(HL),
            0xe6 => self.and_imm(),
            0xe7 => self.rst(0x20),
            0xe8 => self.add_sp_n(),
            0xe9 => self.jp_hl(),
            0xea => self.ld_imm16_a(),
            0xee => self.xor_imm(),
            0xef => self.rst(0x28),
            0xf0 => self.ldh_a_n(),
            0xf1 => self.pop_nn(AF),
            0xf3 => self.ime = false,
            0xf5 => self.push_nn(AF),
            0xf6 => self.or_imm(),
            0xf7 => self.rst(0x30),
            0xf8 => self.ld_hl_sp_n(),
            0xf9 => self.ld_sp_hl(),
            0xfa => self.ld_a_imm16(),
            0xfb => self.ime = true,
            0xfe => self.cp_imm(),
            0xff => self.rst(0x38),
            _ => panic!("ERROR: unknown instruction 0x{:02x}", opcode)
        }

        //if pc >= 0x100 {
        //    println!("AF:{:04x} BC:{:04x} DE:{:04x} HL:{:04x} SP:{:04x}", self.regs.read16(AF), self.regs.read16(BC), self.regs.read16(DE), self.regs.read16(HL), self.regs.read16(SP));
        //}
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

    fn cpl(&mut self) {
        self.regs.a ^= 0xff;

        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, true);
    }

    fn ld_nn_sp(&mut self) {
        let pc = self.imm16();
        let address = self.read16(pc);
        let sp = self.regs.read16(SP);

        self.write16(address, sp);
    }

    fn add_sp_n(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc) as i8 as u16;
        let sp = self.regs.sp;

        self.regs.sp = sp.wrapping_add(value);

        self.regs.f.set(Flags::ZERO, false);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (sp & 0x0f) + (value & 0x0f) > 0x0f);
        self.regs.f.set(Flags::CARRY, (sp & 0xff) + (value & 0xff) > 0xff);
    }

    fn ld_sp_hl(&mut self) {
        self.regs.sp = self.regs.read16(HL);
    }

    fn ld_hl_sp_n(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc) as i8 as u16;
        let sp = self.regs.sp;

        let result = sp.wrapping_add(value);

        self.regs.write16(HL, result);
        self.regs.f.set(Flags::ZERO, false);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (sp & 0x0f) + (value & 0x0f) > 0x0f);
        self.regs.f.set(Flags::CARRY, (sp & 0xff) + (value & 0xff) > 0xff);
    }

    fn rst(&mut self, address: u16) {
        let pc = self.regs.pc;
        self.push16(pc);

        self.regs.pc = address;
    }

    fn jp(&mut self) {
        let pc = self.imm16();
        self.regs.pc = self.read16(pc);
    }

    fn jp_cond(&mut self, cond: Cond) {
        let pc = self.imm16();

        if self.condition_met(cond) {
            self.regs.pc = self.read16(pc);
        }
    }

    fn jp_hl(&mut self) {
        self.regs.pc = self.regs.read16(HL);
    }

    fn add_hl_nn(&mut self, reg: Reg16) {
        let hl = self.regs.read16(HL);
        let value = self.regs.read16(reg);

        let result = hl.wrapping_add(value);

        self.regs.write16(HL, result);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (hl & 0xfff) + (value & 0xfff) > 0xfff);
        self.regs.f.set(Flags::CARRY, (hl as usize) + (value as usize) > 0xffff);
    }

    fn adc_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let a = self.regs.a;

        let result = a.wrapping_add(value).wrapping_add(carry);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) + (value & 0x0f) + carry > 0x0f);
        self.regs.f.set(Flags::CARRY, (a as usize) + (value as usize) + (carry as usize) > 0xff);
    }

    fn sbc_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let a = self.regs.a;

        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f) + carry);
        self.regs.f.set(Flags::CARRY, (a as usize) < (value as usize) + (carry as usize));
    }

    fn and_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);

        self.regs.a &= value;

        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, true);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn or_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);

        self.regs.a |= value;

        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn xor_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);

        self.regs.a ^= value;

        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn add(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);
        let a = self.regs.a;

        let result = a.wrapping_add(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) + (value & 0x0f) > 0x0f);
        self.regs.f.set(Flags::CARRY, (a as usize) + (value as usize) > 0xff);
    }

    fn add_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);
        let a = self.regs.a;

        let result = a.wrapping_add(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) + (value & 0x0f) > 0x0f);
        self.regs.f.set(Flags::CARRY, (a as usize) + (value as usize) > 0xff);
    }

    fn sub_imm(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc);
        let a = self.regs.a;

        let result = a.wrapping_sub(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f));
        self.regs.f.set(Flags::CARRY, a < value);
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

    fn cp(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);
        let a = self.regs.a;

        self.regs.f.set(Flags::ZERO, a == value);
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

    fn dec_hlp(&mut self) {
        let address = self.regs.read16(HL);
        let value = self.read8(address);

        self.write8(address, value.wrapping_sub(1));
        self.regs.f.set(Flags::ZERO, value == 0x1);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (value & 0x0f) == 0);
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

    fn condition_met(&self, cond: Cond) -> bool {
        match cond {
            Cond::C => self.regs.f.contains(Flags::CARRY),
            Cond::NC => !self.regs.f.contains(Flags::CARRY),
            Cond::Z => self.regs.f.contains(Flags::ZERO),
            Cond::NZ => !self.regs.f.contains(Flags::ZERO),
        }
    }

    fn ret_cond(&mut self, cond: Cond) {
        if self.condition_met(cond) {
            self.regs.pc = self.pop16();
        }
    }

    fn inc(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg).wrapping_add(1);
        
        self.regs.write8(reg, value);
        self.regs.f.set(Flags::ZERO, value == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (value & 0x0f) == 0);
    }

    fn inc16(&mut self, reg: Reg16) {
        let value = self.regs.read16(reg);
        
        self.regs.write16(reg, value.wrapping_add(1));
    }

    fn dec16(&mut self, reg: Reg16) {
        let value = self.regs.read16(reg);
        
        self.regs.write16(reg, value.wrapping_sub(1));
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

    fn ld_nn_a(&mut self, reg: Reg16) {
        let address = self.regs.read16(reg);
        let a = self.regs.a;

        self.write8(address, a);
    }

    fn ld_a_nn(&mut self, reg: Reg16) {
        let address = self.regs.read16(reg);

        self.regs.a = self.read8(address);
    }

    fn ld_a_imm16(&mut self) {
        let pc = self.imm16();
        let address = self.read16(pc);

        self.regs.a = self.read8(address);
    }

    fn ld_imm16_a(&mut self) {
        let pc = self.imm16();
        let address = self.read16(pc);
        let value = self.regs.a;

        self.write8(address, value);
    }

    fn ld_n_hlp(&mut self, reg: Reg8) {
        let hl = self.regs.read16(HL);
        let value = self.read8(hl);

        self.regs.write8(reg, value);
    }

    fn ld_hlp_n(&mut self, reg: Reg8) {
        let hl = self.regs.read16(HL);
        let value = self.regs.read8(reg);

        self.write8(hl, value);
    }

    fn ld_hlp_imm(&mut self) {
        let hl = self.regs.read16(HL);
        let pc = self.imm();
        let value = self.read8(pc);

        self.write8(hl, value);
    }

    fn jr(&mut self) {
        let pc = self.imm();
        let value = self.read8(pc) as i8;

        self.regs.pc = self.regs.pc.wrapping_add(value as u16);
    }

    fn jr_cond(&mut self, cond: Cond) {
        let pc = self.imm();
        let value = self.read8(pc) as i8;

        if self.condition_met(cond) {
            self.regs.pc = self.regs.pc.wrapping_add(value as u16);
        }
    }

    fn ld_imm16(&mut self, reg: Reg16) {
        let pc = self.imm16();
        let value = self.read16(pc);
        self.regs.write16(reg, value);
    }

    fn ld_a_hlpp(&mut self) {
        let hl = self.regs.read16(HL);
        
        self.regs.a = self.read8(hl);
        self.regs.write16(HL, hl.wrapping_add(1));
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

    fn and(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        self.regs.a &= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, true);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn or(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        self.regs.a |= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn or_hlp(&mut self) {
        let hl = self.regs.read16(HL);
        let value = self.read8(hl);

        self.regs.a |= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn xor(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        self.regs.a ^= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn xor_hlp(&mut self) {
        let hl = self.regs.read16(HL);
        let value = self.read8(hl);

        self.regs.a ^= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn cb_instr(&mut self) {
        let pc = self.imm();
        let opcode = self.read8(pc);

        match opcode {
            0x11 => self.alu_rl(C),
            0x18 => self.alu_rr(B),
            0x19 => self.alu_rr(C),
            0x1a => self.alu_rr(D),
            0x1b => self.alu_rr(E),
            0x1c => self.alu_rr(H),
            0x1d => self.alu_rr(L),
            0x1f => self.alu_rr(A),
            0x37 => self.cb_swap(A),
            0x38 => self.cb_srl(B),
            0x7c => self.cb_bit(7, H),
            0x87 => self.cb_res(0, A),
            _ => panic!("ERROR: unknown cb instruction 0x{:02x}", opcode)
        }
    }

    fn rla(&mut self) {
        self.alu_rl(A);
        self.regs.f.set(Flags::ZERO, false);
    }

    fn rlca(&mut self) {
        self.alu_rlc(A);
        self.regs.f.set(Flags::ZERO, false);
    }

    fn rra(&mut self) {
        self.alu_rr(A);
        self.regs.f.set(Flags::ZERO, false);
    }

    fn rrca(&mut self) {
        self.alu_rrc(A);
        self.regs.f.set(Flags::ZERO, false);
    }

    fn alu_rl(&mut self, reg: Reg8) {
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let value = self.regs.read8(reg);

        let result = (value << 1) | carry;

        self.regs.write8(reg, result);
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x80) != 0);
    }

    fn alu_rlc(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        let result = (value << 1) | (value >> 7);

        self.regs.write8(reg, result);
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x80) != 0);
    }

    fn alu_rr(&mut self, reg: Reg8) {
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let value = self.regs.read8(reg);

        let result = (carry << 7) | (value >> 1);

        self.regs.write8(reg, result);
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x01) != 0);
    }

    fn alu_rrc(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        let result = (value >> 1) | (value << 7);

        self.regs.write8(reg, result);
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x01) != 0);
    }

    fn cb_swap(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        let result = (value >> 4) | (value << 4);

        self.regs.write8(reg, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn cb_res(&mut self, bit: u8, reg: Reg8) {
        let value = self.regs.read8(reg);

        assert!(bit <= 7);

        self.regs.write8(reg, value & !(1 << bit));
    }

    fn cb_srl(&mut self, reg: Reg8) {
        let value = self.regs.read8(reg);

        let result = value >> 1;

        self.regs.write8(reg, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x1) != 0);
    }

    fn cb_bit(&mut self, bit: usize, reg: Reg8) {
        let value = self.regs.read8(reg);

        assert!(bit <= 7);

        self.regs.f.set(Flags::ZERO, (value & (1 << bit)) == 0);
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
        let sp = self.regs.sp;
        self.regs.sp += 1;
        
        self.read8(sp)
    }

    fn pop16(&mut self) -> u16 {
        (self.pop8() as u16) | ((self.pop8() as u16) << 8)
    }

    fn push8(&mut self, value: u8) {
        self.regs.sp -= 1;
        let sp = self.regs.sp;

        self.write8(sp, value);
    }

    fn push16(&mut self, value: u16) {
        self.push8((value >> 8) as u8);
        self.push8(value as u8);
    }

    fn call_cond(&mut self, cond: Cond) {
        let pc = self.imm16();

        if self.condition_met(cond) {
            self.push16(pc + 2);
            self.regs.pc = self.read16(pc);
        }
    }

    fn call(&mut self) {
        let pc = self.imm16();
        self.push16(pc + 2);
        self.regs.pc = self.read16(pc);
    }
}
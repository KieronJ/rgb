mod registers;

use self::registers::{Registers, Flags, Reg16};
use self::registers::Reg16::{AF, BC, DE, HL, SP};

use super::bus;

#[derive(Clone, Copy)]
enum AddressingMode {
    A, B, C, D, E, H, L,
    HLP,
    IMM,
}

use self::AddressingMode::{A, B, C, D, E, H, L, HLP, IMM};

#[derive(Clone, Copy, PartialEq)]
enum Cond {
    NONE,
    C, NC,
    Z, NZ
}

pub struct Z80 {
    bus: bus::Bus,
    regs: Registers,
    ime: bool,
    halt: bool,
}

impl Z80 {
    pub fn new(bus: bus::Bus) -> Z80 {
        Z80 {
            bus: bus,
            regs: Registers::new(),
            ime: false,
            halt: false,
        }
    }

    pub fn reset(&mut self) {
        self.regs.pc = 0;
        self.ime = true;
        self.halt = false;
    }

    fn interrupt(&mut self, address: u16) {
        self.ime = false;

        self.bus.tick();
        self.bus.tick();

        let pc = self.regs.pc;
        self.push16(pc);

        self.regs.pc = address;
    }

    fn check_interrupts(&mut self) -> bool {
        let _ie = self.bus.read(0xffff);
        let _if = self.bus.read(0xff0f);

        let interrupt_status = _ie & _if;

        let interrupt = interrupt_status != 0;

        if self.ime {
            if (interrupt_status & 0x01) != 0 {
                self.write8(0xff0f, _if & !0x01);
                self.interrupt(0x40);
            }

            else if (interrupt_status & 0x02) != 0 {
                self.write8(0xff0f, _if & !0x02);
                self.interrupt(0x48);
            }

            else if (interrupt_status & 0x04) != 0 {
                self.write8(0xff0f, _if & !0x04);
                self.interrupt(0x50);
            }

            else if (interrupt_status & 0x10) != 0 {
                self.write8(0xff0f, _if & !0x10);
                self.interrupt(0x60);
            }
        }

        interrupt
    }

    pub fn run(&mut self) {
        if self.check_interrupts() {
            self.bus.tick();
            self.halt = false;
        }

        if self.halt {
            self.bus.tick();
        } else {
            self.execute_instruction();
        }
    }

    fn execute_instruction(&mut self) {
        let pc = self.imm();
        let opcode = self.read8(pc);

        match opcode {
            0x00 => {}, // nop
            0x01 => self.ld_imm16(BC),
            0x02 => self.ld_nn_a(BC),
            0x03 => self.inc16(BC),
            0x04 => self.inc(B),
            0x05 => self.dec(B),
            0x06 => self.ld_imm(B),
            0x07 => self.rlca(),
            0x08 => self.ld_nn_sp(),
            0x09 => self.add_hl_nn(BC),
            0x0a => self.ld_a_nn(BC),
            0x0b => self.dec16(BC),
            0x0c => self.inc(C),
            0x0d => self.dec(C),
            0x0e => self.ld_imm(C),
            0x0f => self.rrca(),
            0x11 => self.ld_imm16(DE),
            0x12 => self.ld_nn_a(DE),
            0x13 => self.inc16(DE),
            0x14 => self.inc(D),
            0x15 => self.dec(D),
            0x16 => self.ld_imm(D),
            0x17 => self.rla(),
            0x18 => self.jr(Cond::NONE),
            0x19 => self.add_hl_nn(DE),
            0x1a => self.ld_a_nn(DE),
            0x1b => self.dec16(DE),
            0x1c => self.inc(E),
            0x1d => self.dec(E),
            0x1e => self.ld_imm(E),
            0x1f => self.rra(),
            0x20 => self.jr(Cond::NZ),
            0x21 => self.ld_imm16(HL),
            0x22 => self.ld_hlpp_a(),
            0x23 => self.inc16(HL),
            0x24 => self.inc(H),
            0x25 => self.dec(H),
            0x26 => self.ld_imm(H),
            0x27 => self.daa(),
            0x28 => self.jr(Cond::Z),
            0x29 => self.add_hl_nn(HL),
            0x2a => self.ld_a_hlpp(),
            0x2b => self.dec16(HL),
            0x2c => self.inc(L),
            0x2d => self.dec(L),
            0x2e => self.ld_imm(L),
            0x2f => self.cpl(),
            0x30 => self.jr(Cond::NC),
            0x31 => self.ld_imm16(SP),
            0x32 => self.ld_hlmp_a(),
            0x33 => self.inc16(SP),
            0x34 => self.inc(HLP),
            0x35 => self.dec(HLP),
            0x36 => self.ld_hlp_imm(),
            0x37 => self.scf(),
            0x38 => self.jr(Cond::C),
            0x39 => self.add_hl_nn(SP),
            0x3a => self.ld_a_hlmp(),
            0x3b => self.dec16(SP),
            0x3c => self.inc(A),
            0x3d => self.dec(A),
            0x3e => self.ld_imm(A),
            0x3f => self.ccf(),
            0x40 => self.ld_n_n(B, B),
            0x41 => self.ld_n_n(B, C),
            0x42 => self.ld_n_n(B, D),
            0x43 => self.ld_n_n(B, E),
            0x44 => self.ld_n_n(B, H),
            0x45 => self.ld_n_n(B, L),
            0x46 => self.ld_n_n(B, HLP),
            0x47 => self.ld_n_n(B, A),
            0x48 => self.ld_n_n(C, B),
            0x49 => self.ld_n_n(C, C),
            0x4a => self.ld_n_n(C, D),
            0x4b => self.ld_n_n(C, E),
            0x4c => self.ld_n_n(C, H),
            0x4d => self.ld_n_n(C, L),
            0x4e => self.ld_n_n(C, HLP),
            0x4f => self.ld_n_n(C, A),
            0x50 => self.ld_n_n(D, B),
            0x51 => self.ld_n_n(D, C),
            0x52 => self.ld_n_n(D, D),
            0x53 => self.ld_n_n(D, E),
            0x54 => self.ld_n_n(D, H),
            0x55 => self.ld_n_n(D, L),
            0x56 => self.ld_n_n(D, HLP),
            0x57 => self.ld_n_n(D, A),
            0x58 => self.ld_n_n(E, B),
            0x59 => self.ld_n_n(E, C),
            0x5a => self.ld_n_n(E, D),
            0x5b => self.ld_n_n(E, E),
            0x5c => self.ld_n_n(E, H),
            0x5d => self.ld_n_n(E, L),
            0x5e => self.ld_n_n(E, HLP),
            0x5f => self.ld_n_n(E, A),
            0x60 => self.ld_n_n(H, B),
            0x61 => self.ld_n_n(H, C),
            0x62 => self.ld_n_n(H, D),
            0x63 => self.ld_n_n(H, E),
            0x64 => self.ld_n_n(H, H),
            0x65 => self.ld_n_n(H, L),
            0x66 => self.ld_n_n(H, HLP),
            0x67 => self.ld_n_n(H, A),
            0x68 => self.ld_n_n(L, B),
            0x69 => self.ld_n_n(L, C),
            0x6a => self.ld_n_n(L, D),
            0x6b => self.ld_n_n(L, E),
            0x6c => self.ld_n_n(L, H),
            0x6d => self.ld_n_n(L, L),
            0x6e => self.ld_n_n(L, HLP),
            0x6f => self.ld_n_n(L, A),
            0x70 => self.ld_n_n(HLP, B),
            0x71 => self.ld_n_n(HLP, C),
            0x72 => self.ld_n_n(HLP, D),
            0x73 => self.ld_n_n(HLP, E),
            0x74 => self.ld_n_n(HLP, H),
            0x75 => self.ld_n_n(HLP, L),
            0x76 => self.halt(),
            0x77 => self.ld_n_n(HLP, A),
            0x78 => self.ld_n_n(A, B),
            0x79 => self.ld_n_n(A, C),
            0x7a => self.ld_n_n(A, D),
            0x7b => self.ld_n_n(A, E),
            0x7c => self.ld_n_n(A, H),
            0x7d => self.ld_n_n(A, L),
            0x7e => self.ld_n_n(A, HLP),
            0x7f => self.ld_n_n(A, A),
            0x80 => self.add(B),
            0x81 => self.add(C),
            0x82 => self.add(D),
            0x83 => self.add(E),
            0x84 => self.add(H),
            0x85 => self.add(L),
            0x86 => self.add(HLP),
            0x87 => self.add(A),
            0x88 => self.adc(B),
            0x89 => self.adc(C),
            0x8a => self.adc(D),
            0x8b => self.adc(E),
            0x8c => self.adc(H),
            0x8d => self.adc(L),
            0x8e => self.adc(HLP),
            0x8f => self.adc(A),
            0x90 => self.sub(B),
            0x91 => self.sub(C),
            0x92 => self.sub(D),
            0x93 => self.sub(E),
            0x94 => self.sub(H),
            0x95 => self.sub(L),
            0x96 => self.sub(HLP),
            0x97 => self.sub(A),
            0x98 => self.sbc(B),
            0x99 => self.sbc(C),
            0x9a => self.sbc(D),
            0x9b => self.sbc(E),
            0x9c => self.sbc(H),
            0x9d => self.sbc(L),
            0x9e => self.sbc(HLP),
            0x9f => self.sbc(A),
            0xa0 => self.and(B),
            0xa1 => self.and(C),
            0xa2 => self.and(D),
            0xa3 => self.and(E),
            0xa4 => self.and(H),
            0xa5 => self.and(L),
            0xa6 => self.and(HLP),
            0xa7 => self.and(A),
            0xa8 => self.xor(B),
            0xa9 => self.xor(C),
            0xaa => self.xor(D),
            0xab => self.xor(E),
            0xac => self.xor(H),
            0xad => self.xor(L),
            0xae => self.xor(HLP),
            0xaf => self.xor(A),
            0xb0 => self.or(B),
            0xb1 => self.or(C),
            0xb2 => self.or(D),
            0xb3 => self.or(E),
            0xb4 => self.or(H),
            0xb5 => self.or(L),
            0xb6 => self.or(HLP),
            0xb7 => self.or(A),
            0xb8 => self.cp(B),
            0xb9 => self.cp(C),
            0xba => self.cp(D),
            0xbb => self.cp(E),
            0xbc => self.cp(H),
            0xbd => self.cp(L),
            0xbe => self.cp(HLP),
            0xbf => self.cp(A),
            0xc0 => self.ret(Cond::NZ),
            0xc1 => self.pop_nn(BC),
            0xc2 => self.jp(Cond::NZ),
            0xc3 => self.jp(Cond::NONE),
            0xc4 => self.call(Cond::NZ),
            0xc5 => self.push_nn(BC),
            0xc6 => self.add(IMM),
            0xc7 => self.rst(0x00),
            0xc8 => self.ret(Cond::Z),
            0xc9 => self.ret(Cond::NONE),
            0xca => self.jp(Cond::Z),
            0xcb => self.cb_instr(),
            0xcc => self.call(Cond::Z),
            0xcd => self.call(Cond::NONE),
            0xce => self.adc(IMM),
            0xcf => self.rst(0x08),
            0xd0 => self.ret(Cond::NC),
            0xd1 => self.pop_nn(DE),
            0xd2 => self.jp(Cond::NC),
            0xd4 => self.call(Cond::NC),
            0xd5 => self.push_nn(DE),
            0xd6 => self.sub(IMM),
            0xd7 => self.rst(0x10),
            0xd8 => self.ret(Cond::C),
            0xd9 => self.reti(),
            0xda => self.jp(Cond::C),
            0xdc => self.call(Cond::C),
            0xde => self.sbc(IMM),
            0xdf => self.rst(0x18),
            0xe0 => self.ldh_n_a(),
            0xe1 => self.pop_nn(HL),
            0xe2 => self.ld_cp_a(),
            0xe5 => self.push_nn(HL),
            0xe6 => self.and(IMM),
            0xe7 => self.rst(0x20),
            0xe8 => self.add_sp_n(),
            0xe9 => self.jp_hl(),
            0xea => self.ld_imm16_a(),
            0xee => self.xor(IMM),
            0xef => self.rst(0x28),
            0xf0 => self.ldh_a_n(),
            0xf1 => self.pop_nn(AF),
            0xf2 => self.ld_a_cp(),
            0xf3 => self.ime = false,
            0xf5 => self.push_nn(AF),
            0xf6 => self.or(IMM),
            0xf7 => self.rst(0x30),
            0xf8 => self.ld_hl_sp_n(),
            0xf9 => self.ld_sp_hl(),
            0xfa => self.ld_a_imm16(),
            0xfb => self.ime = true,
            0xfe => self.cp(IMM),
            0xff => self.rst(0x38),
            _ => panic!("ERROR: unknown instruction 0x{:02x}", opcode)
        }
    }

    fn read8(&mut self, address: u16) -> u8 {
        self.bus.tick();
        self.bus.read(address)
    }

    fn read16(&mut self, address: u16) -> u16 {
        (self.read8(address) as u16) | ((self.read8(address + 1) as u16) << 8)
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.bus.tick();
        self.bus.write(address, value);
    }

    fn write16(&mut self, address: u16, value: u16) {
        self.write8(address, value as u8);
        self.write8(address + 1, (value >> 8) as u8);
    }

    fn imm(&mut self) -> u16 {
        self.regs.pc += 1;
        self.regs.pc - 1
    }

    fn imm16(&mut self) -> u16 {
        self.regs.pc += 2;
        self.regs.pc - 2
    }

    fn read_mode(&mut self, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::A => self.regs.a,
            AddressingMode::B => self.regs.b,
            AddressingMode::C => self.regs.c,
            AddressingMode::D => self.regs.d,
            AddressingMode::E => self.regs.e,
            AddressingMode::H => self.regs.h,
            AddressingMode::L => self.regs.l,
            AddressingMode::HLP => {
                let hl = self.regs.read16(HL);
                self.read8(hl)
            },
            AddressingMode::IMM => {
                let imm = self.imm();
                self.read8(imm)
            },
        }
    }

    fn write_mode(&mut self, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::A => self.regs.a = value,
            AddressingMode::B => self.regs.b = value,
            AddressingMode::C => self.regs.c = value,
            AddressingMode::D => self.regs.d = value,
            AddressingMode::E => self.regs.e = value,
            AddressingMode::H => self.regs.h = value,
            AddressingMode::L => self.regs.l = value,
            AddressingMode::HLP => {
                let hl = self.regs.read16(HL);
                self.write8(hl, value);
            },
            AddressingMode::IMM => panic!("ERROR: write to IMM AddressingMode"),
        }
    }

    fn halt(&mut self) {
        self.halt = true;
    }

    fn cpl(&mut self) {
        self.regs.a ^= 0xff;

        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, true);
    }

    fn scf(&mut self) {
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, true);
    }

    fn ccf(&mut self) {
        let carry = self.regs.f.contains(Flags::CARRY);

        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, !carry);
    }

    fn daa(&mut self) {
        let mut carry = false;

        if !self.regs.f.contains(Flags::SUBTRACT) {
            if self.regs.f.contains(Flags::CARRY) || (self.regs.a > 0x99) {
                self.regs.a = self.regs.a.wrapping_add(0x60);
                carry = true;
            }

            if self.regs.f.contains(Flags::HALFCARRY) || ((self.regs.a & 0x0f) > 0x09) {
                self.regs.a = self.regs.a.wrapping_add(0x06);
            }
        } else {
            if self.regs.f.contains(Flags::CARRY) {
                self.regs.a = self.regs.a.wrapping_sub(0x60);
                carry = true;
            }

            if self.regs.f.contains(Flags::HALFCARRY) {
                self.regs.a = self.regs.a.wrapping_sub(0x06);
            }
        }

        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, carry);
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

        self.bus.tick();
        self.bus.tick();
    }

    fn ld_sp_hl(&mut self) {
        self.regs.sp = self.regs.read16(HL);
        
        self.bus.tick();
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

        self.bus.tick();
    }

    fn rst(&mut self, address: u16) {
        let pc = self.regs.pc;
        self.push16(pc);

        self.regs.pc = address;

        self.bus.tick();
    }

    fn jp(&mut self, cond: Cond) {
        let pc = self.imm16();
        let dest = self.read16(pc);

        if self.condition_met(cond) {
            self.regs.pc = dest;

            self.bus.tick();
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

        self.bus.tick();
    }

    fn adc(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let a = self.regs.a;

        let result = a.wrapping_add(value).wrapping_add(carry);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) + (value & 0x0f) + carry > 0x0f);
        self.regs.f.set(Flags::CARRY, (a as usize) + (value as usize) + (carry as usize) > 0xff);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let a = self.regs.a;

        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f) + carry);
        self.regs.f.set(Flags::CARRY, (a as usize) < (value as usize) + (carry as usize));
    }

    fn add(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);
        let a = self.regs.a;

        let result = a.wrapping_add(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) + (value & 0x0f) > 0x0f);
        self.regs.f.set(Flags::CARRY, (a as usize) + (value as usize) > 0xff);
    }

    fn sub(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);
        let a = self.regs.a;

        let result = a.wrapping_sub(value);

        self.regs.a = result;
        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (a & 0x0f) < (value & 0x0f));
        self.regs.f.set(Flags::CARRY, a < value);
    }

    fn cp(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);
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
        self.bus.tick();

        let value = self.regs.read16(reg);
        self.push16(value);
    }

    fn ld_n_n(&mut self, dest: AddressingMode, src: AddressingMode) {
        let value = self.read_mode(src);
        self.write_mode(dest, value);
    }

    fn ret(&mut self, cond: Cond) {
        if cond != Cond::NONE {
            self.bus.tick();
        }

        if self.condition_met(cond) {
            self.regs.pc = self.pop16();

            self.bus.tick();
        }
    }

    fn reti(&mut self) {
        self.ime = true;
        self.regs.pc = self.pop16();

        self.bus.tick();
    }

    fn condition_met(&self, cond: Cond) -> bool {
        match cond {
            Cond::NONE => true,
            Cond::C => self.regs.f.contains(Flags::CARRY),
            Cond::NC => !self.regs.f.contains(Flags::CARRY),
            Cond::Z => self.regs.f.contains(Flags::ZERO),
            Cond::NZ => !self.regs.f.contains(Flags::ZERO),
        }
    }

    fn inc(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = value.wrapping_add(1);
        
        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, (value & 0x0f) == 0x0f);
    }

    fn inc16(&mut self, reg: Reg16) {
        let value = self.regs.read16(reg);

        self.regs.write16(reg, value.wrapping_add(1));

        self.bus.tick();
    }

    fn dec16(&mut self, reg: Reg16) {
        let value = self.regs.read16(reg);
        
        self.regs.write16(reg, value.wrapping_sub(1));

        self.bus.tick();
    }

    fn dec(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);
        
        let result = value.wrapping_sub(1);

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, true);
        self.regs.f.set(Flags::HALFCARRY, (value & 0x0f) == 0);
    }

    fn ld_imm(&mut self, mode: AddressingMode) {
        let pc = self.imm();
        let value = self.read8(pc);
        self.write_mode(mode, value);
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

    fn ld_hlp_imm(&mut self) {
        let hl = self.regs.read16(HL);
        let pc = self.imm();
        let value = self.read8(pc);

        self.write8(hl, value);
    }

    fn jr(&mut self, cond: Cond) {
        let pc = self.imm();
        let value = self.read8(pc) as i8;

        if self.condition_met(cond) {
            self.regs.pc = self.regs.pc.wrapping_add(value as u16);

            self.bus.tick();
        }
    }

    fn ld_imm16(&mut self, reg: Reg16) {
        let pc = self.imm16();
        let value = self.read16(pc);
        self.regs.write16(reg, value);
    }

    fn ld_a_hlmp(&mut self) {
        let hl = self.regs.read16(HL);
        
        self.regs.a = self.read8(hl);
        self.regs.write16(HL, hl.wrapping_sub(1));
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

    fn and(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        self.regs.a &= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, true);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn or(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        self.regs.a |= value;
        self.regs.f.set(Flags::ZERO, self.regs.a == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn xor(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

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
            0x00 => self.alu_rlc(B),
            0x01 => self.alu_rlc(C),
            0x02 => self.alu_rlc(D),
            0x03 => self.alu_rlc(E),
            0x04 => self.alu_rlc(H),
            0x05 => self.alu_rlc(L),
            0x06 => self.alu_rlc(HLP),
            0x07 => self.alu_rlc(A),
            0x08 => self.alu_rrc(B),
            0x09 => self.alu_rrc(C),
            0x0a => self.alu_rrc(D),
            0x0b => self.alu_rrc(E),
            0x0c => self.alu_rrc(H),
            0x0d => self.alu_rrc(L),
            0x0e => self.alu_rrc(HLP),
            0x0f => self.alu_rrc(A),
            0x10 => self.alu_rl(B),
            0x11 => self.alu_rl(C),
            0x12 => self.alu_rl(D),
            0x13 => self.alu_rl(E),
            0x14 => self.alu_rl(H),
            0x15 => self.alu_rl(L),
            0x16 => self.alu_rl(HLP),
            0x17 => self.alu_rl(A),
            0x18 => self.alu_rr(B),
            0x19 => self.alu_rr(C),
            0x1a => self.alu_rr(D),
            0x1b => self.alu_rr(E),
            0x1c => self.alu_rr(H),
            0x1d => self.alu_rr(L),
            0x1e => self.alu_rr(HLP),
            0x1f => self.alu_rr(A),
            0x20 => self.cb_sla(B),
            0x21 => self.cb_sla(C),
            0x22 => self.cb_sla(D),
            0x23 => self.cb_sla(E),
            0x24 => self.cb_sla(H),
            0x25 => self.cb_sla(L),
            0x26 => self.cb_sla(HLP),
            0x27 => self.cb_sla(A),
            0x28 => self.cb_sra(B),
            0x29 => self.cb_sra(C),
            0x2a => self.cb_sra(D),
            0x2b => self.cb_sra(E),
            0x2c => self.cb_sra(H),
            0x2d => self.cb_sra(L),
            0x2e => self.cb_sra(HLP),
            0x2f => self.cb_sra(A),
            0x30 => self.cb_swap(B),
            0x31 => self.cb_swap(C),
            0x32 => self.cb_swap(D),
            0x33 => self.cb_swap(E),
            0x34 => self.cb_swap(H),
            0x35 => self.cb_swap(L),
            0x36 => self.cb_swap(HLP),
            0x37 => self.cb_swap(A),
            0x38 => self.cb_srl(B),
            0x39 => self.cb_srl(C),
            0x3a => self.cb_srl(D),
            0x3b => self.cb_srl(E),
            0x3c => self.cb_srl(H),
            0x3d => self.cb_srl(L),
            0x3e => self.cb_srl(HLP),
            0x3f => self.cb_srl(A),
            0x40 => self.cb_bit(0, B),
            0x41 => self.cb_bit(0, C),
            0x42 => self.cb_bit(0, D),
            0x43 => self.cb_bit(0, E),
            0x44 => self.cb_bit(0, H),
            0x45 => self.cb_bit(0, L),
            0x46 => self.cb_bit(0, HLP),
            0x47 => self.cb_bit(0, A),
            0x48 => self.cb_bit(1, B),
            0x49 => self.cb_bit(1, C),
            0x4a => self.cb_bit(1, D),
            0x4b => self.cb_bit(1, E),
            0x4c => self.cb_bit(1, H),
            0x4d => self.cb_bit(1, L),
            0x4e => self.cb_bit(1, HLP),
            0x4f => self.cb_bit(1, A),
            0x50 => self.cb_bit(2, B),
            0x51 => self.cb_bit(2, C),
            0x52 => self.cb_bit(2, D),
            0x53 => self.cb_bit(2, E),
            0x54 => self.cb_bit(2, H),
            0x55 => self.cb_bit(2, L),
            0x56 => self.cb_bit(2, HLP),
            0x57 => self.cb_bit(2, A),
            0x58 => self.cb_bit(3, B),
            0x59 => self.cb_bit(3, C),
            0x5a => self.cb_bit(3, D),
            0x5b => self.cb_bit(3, E),
            0x5c => self.cb_bit(3, H),
            0x5d => self.cb_bit(3, L),
            0x5e => self.cb_bit(3, HLP),
            0x5f => self.cb_bit(3, A),
            0x60 => self.cb_bit(4, B),
            0x61 => self.cb_bit(4, C),
            0x62 => self.cb_bit(4, D),
            0x63 => self.cb_bit(4, E),
            0x64 => self.cb_bit(4, H),
            0x65 => self.cb_bit(4, L),
            0x66 => self.cb_bit(4, HLP),
            0x67 => self.cb_bit(4, A),
            0x68 => self.cb_bit(5, B),
            0x69 => self.cb_bit(5, C),
            0x6a => self.cb_bit(5, D),
            0x6b => self.cb_bit(5, E),
            0x6c => self.cb_bit(5, H),
            0x6d => self.cb_bit(5, L),
            0x6e => self.cb_bit(5, HLP),
            0x6f => self.cb_bit(5, A),
            0x70 => self.cb_bit(6, B),
            0x71 => self.cb_bit(6, C),
            0x72 => self.cb_bit(6, D),
            0x73 => self.cb_bit(6, E),
            0x74 => self.cb_bit(6, H),
            0x75 => self.cb_bit(6, L),
            0x76 => self.cb_bit(6, HLP),
            0x77 => self.cb_bit(6, A),
            0x78 => self.cb_bit(7, B),
            0x79 => self.cb_bit(7, C),
            0x7a => self.cb_bit(7, D),
            0x7b => self.cb_bit(7, E),
            0x7c => self.cb_bit(7, H),
            0x7d => self.cb_bit(7, L),
            0x7e => self.cb_bit(7, HLP),
            0x7f => self.cb_bit(7, A),
            0x80 => self.cb_res(0, B),
            0x81 => self.cb_res(0, C),
            0x82 => self.cb_res(0, D),
            0x83 => self.cb_res(0, E),
            0x84 => self.cb_res(0, H),
            0x85 => self.cb_res(0, L),
            0x86 => self.cb_res(0, HLP),
            0x87 => self.cb_res(0, A),
            0x88 => self.cb_res(1, B),
            0x89 => self.cb_res(1, C),
            0x8a => self.cb_res(1, D),
            0x8b => self.cb_res(1, E),
            0x8c => self.cb_res(1, H),
            0x8d => self.cb_res(1, L),
            0x8e => self.cb_res(1, HLP),
            0x8f => self.cb_res(1, A),
            0x90 => self.cb_res(2, B),
            0x91 => self.cb_res(2, C),
            0x92 => self.cb_res(2, D),
            0x93 => self.cb_res(2, E),
            0x94 => self.cb_res(2, H),
            0x95 => self.cb_res(2, L),
            0x96 => self.cb_res(2, HLP),
            0x97 => self.cb_res(2, A),
            0x98 => self.cb_res(3, B),
            0x99 => self.cb_res(3, C),
            0x9a => self.cb_res(3, D),
            0x9b => self.cb_res(3, E),
            0x9c => self.cb_res(3, H),
            0x9d => self.cb_res(3, L),
            0x9e => self.cb_res(3, HLP),
            0x9f => self.cb_res(3, A),
            0xa0 => self.cb_res(4, B),
            0xa1 => self.cb_res(4, C),
            0xa2 => self.cb_res(4, D),
            0xa3 => self.cb_res(4, E),
            0xa4 => self.cb_res(4, H),
            0xa5 => self.cb_res(4, L),
            0xa6 => self.cb_res(4, HLP),
            0xa7 => self.cb_res(4, A),
            0xa8 => self.cb_res(5, B),
            0xa9 => self.cb_res(5, C),
            0xaa => self.cb_res(5, D),
            0xab => self.cb_res(5, E),
            0xac => self.cb_res(5, H),
            0xad => self.cb_res(5, L),
            0xae => self.cb_res(5, HLP),
            0xaf => self.cb_res(5, A),
            0xb0 => self.cb_res(6, B),
            0xb1 => self.cb_res(6, C),
            0xb2 => self.cb_res(6, D),
            0xb3 => self.cb_res(6, E),
            0xb4 => self.cb_res(6, H),
            0xb5 => self.cb_res(6, L),
            0xb6 => self.cb_res(6, HLP),
            0xb7 => self.cb_res(6, A),
            0xb8 => self.cb_res(7, B),
            0xb9 => self.cb_res(7, C),
            0xba => self.cb_res(7, D),
            0xbb => self.cb_res(7, E),
            0xbc => self.cb_res(7, H),
            0xbd => self.cb_res(7, L),
            0xbe => self.cb_res(7, HLP),
            0xbf => self.cb_res(7, A),
            0xc0 => self.cb_set(0, B),
            0xc1 => self.cb_set(0, C),
            0xc2 => self.cb_set(0, D),
            0xc3 => self.cb_set(0, E),
            0xc4 => self.cb_set(0, H),
            0xc5 => self.cb_set(0, L),
            0xc6 => self.cb_set(0, HLP),
            0xc7 => self.cb_set(0, A),
            0xc8 => self.cb_set(1, B),
            0xc9 => self.cb_set(1, C),
            0xca => self.cb_set(1, D),
            0xcb => self.cb_set(1, E),
            0xcc => self.cb_set(1, H),
            0xcd => self.cb_set(1, L),
            0xce => self.cb_set(1, HLP),
            0xcf => self.cb_set(1, A),
            0xd0 => self.cb_set(2, B),
            0xd1 => self.cb_set(2, C),
            0xd2 => self.cb_set(2, D),
            0xd3 => self.cb_set(2, E),
            0xd4 => self.cb_set(2, H),
            0xd5 => self.cb_set(2, L),
            0xd6 => self.cb_set(2, HLP),
            0xd7 => self.cb_set(2, A),
            0xd8 => self.cb_set(3, B),
            0xd9 => self.cb_set(3, C),
            0xda => self.cb_set(3, D),
            0xdb => self.cb_set(3, E),
            0xdc => self.cb_set(3, H),
            0xdd => self.cb_set(3, L),
            0xde => self.cb_set(3, HLP),
            0xdf => self.cb_set(3, A),
            0xe0 => self.cb_set(4, B),
            0xe1 => self.cb_set(4, C),
            0xe2 => self.cb_set(4, D),
            0xe3 => self.cb_set(4, E),
            0xe4 => self.cb_set(4, H),
            0xe5 => self.cb_set(4, L),
            0xe6 => self.cb_set(4, HLP),
            0xe7 => self.cb_set(4, A),
            0xe8 => self.cb_set(5, B),
            0xe9 => self.cb_set(5, C),
            0xea => self.cb_set(5, D),
            0xeb => self.cb_set(5, E),
            0xec => self.cb_set(5, H),
            0xed => self.cb_set(5, L),
            0xee => self.cb_set(5, HLP),
            0xef => self.cb_set(5, A),
            0xf0 => self.cb_set(6, B),
            0xf1 => self.cb_set(6, C),
            0xf2 => self.cb_set(6, D),
            0xf3 => self.cb_set(6, E),
            0xf4 => self.cb_set(6, H),
            0xf5 => self.cb_set(6, L),
            0xf6 => self.cb_set(6, HLP),
            0xf7 => self.cb_set(6, A),
            0xf8 => self.cb_set(7, B),
            0xf9 => self.cb_set(7, C),
            0xfa => self.cb_set(7, D),
            0xfb => self.cb_set(7, E),
            0xfc => self.cb_set(7, H),
            0xfd => self.cb_set(7, L),
            0xfe => self.cb_set(7, HLP),
            0xff => self.cb_set(7, A),
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

    fn alu_rl(&mut self, mode: AddressingMode) {
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let value = self.read_mode(mode);

        let result = (value << 1) | carry;

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x80) != 0);
    }

    fn alu_rlc(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = (value << 1) | (value >> 7);

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x80) != 0);
    }

    fn alu_rr(&mut self, mode: AddressingMode) {
        let carry = self.regs.f.contains(Flags::CARRY) as u8;
        let value = self.read_mode(mode);

        let result = (carry << 7) | (value >> 1);

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x01) != 0);
    }

    fn alu_rrc(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = (value >> 1) | (value << 7);

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x01) != 0);
    }

    fn cb_sla(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = value << 1;
        
        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x80) != 0);
    }

    fn cb_sra(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = (value & 0x80) | (value >> 1);

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x01) != 0);
    }

    fn cb_swap(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = (value >> 4) | (value << 4);

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, false);
    }

    fn cb_res(&mut self, bit: u8, mode: AddressingMode) {
        let value = self.read_mode(mode);

        assert!(bit <= 7);

        self.write_mode(mode, value & !(1 << bit));
    }

    fn cb_set(&mut self, bit: u8, mode: AddressingMode) {
        let value = self.read_mode(mode);

        assert!(bit <= 7);

        self.write_mode(mode, value | (1 << bit));
    }

    fn cb_srl(&mut self, mode: AddressingMode) {
        let value = self.read_mode(mode);

        let result = value >> 1;

        self.write_mode(mode, result);

        self.regs.f.set(Flags::ZERO, result == 0);
        self.regs.f.set(Flags::SUBTRACT, false);
        self.regs.f.set(Flags::HALFCARRY, false);
        self.regs.f.set(Flags::CARRY, (value & 0x1) != 0);
    }

    fn cb_bit(&mut self, bit: usize, mode: AddressingMode) {
        let value = self.read_mode(mode);

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

    fn ld_a_cp(&mut self) {
        let address = 0xff00 + self.regs.c as u16;
        self.regs.a = self.read8(address)
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
        let l = self.pop8();
        let h = self.pop8();

        ((h as u16) << 8) | (l as u16)
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

    fn call(&mut self, cond: Cond) {
        let pc = self.imm16();
        let dest = self.read16(pc);

        if self.condition_met(cond) {
            self.bus.tick();

            self.push16(pc + 2);
            self.regs.pc = dest;
        }
    }
}
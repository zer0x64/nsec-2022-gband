mod decoder;

use bitflags::bitflags;

use crate::bus::CpuBus;
use decoder::{Alu, Opcode, OpMemAddress8, OpMemAddress16, Register, RegisterPair};

bitflags! {
    pub struct FlagRegister: u8 {
        const UNUSED = 0x0F;
        const C = 0x10;
        const H = 0x20;
        const N = 0x40;
        const Z = 0x80;
    }
}

pub struct Cpu {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub a: u8,
    pub f: FlagRegister,
    pub sp: u16,
    pub pc: u16,
    pub cycles: u8,

    pub opcode_latch: Opcode
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            a: 0,
            f: FlagRegister::empty(),
            sp: 0,
            pc: 0,
            cycles: 0,

            opcode_latch: Opcode::Unknown
        }
    }
}

impl Cpu {
    pub fn clock(&mut self, bus: &mut CpuBus) {
        // Fetch/Execute overlap, last cycle of execute runs at the same time as the next fetch
        if self.cycles != 0 {
            self.execute(bus);

            // We are not emulating cycle-accurate yet, so just reset the latch to unknown to noop the remaining cycles
            self.opcode_latch = Opcode::Unknown;

            self.cycles -= 1;
        }

        if self.cycles == 0 {
            self.fetch(bus);
        }
    }

    // TODO: Remove pub added for criterion
    pub fn fetch(&mut self, bus: &mut CpuBus) {
        self.opcode_latch = Opcode::from(self.read_immediate(bus));
        self.cycles = self.opcode_latch.cycles();
    }

    // TODO: Remove pub added for criterion
    pub fn execute(&mut self, bus: &mut CpuBus) {
        // In Z80 / GB, unknown instructions are just noop
        match self.opcode_latch {
            Opcode::Unknown => {
                // noop
            }
            Opcode::LdRR(target, source) => {
                self.set_register(target, self.get_register(source));
            }
            Opcode::LdRImm(target) => {
                let immediate = self.read_immediate(bus);
                self.set_register(target, immediate);
            }
            Opcode::LdRMem(target, source) => {
                let val = match source {
                    OpMemAddress16::Register(source) => {
                        bus.read_ram(self.get_register_pair(source))
                    }
                    OpMemAddress16::RegisterIncrease(source) => {
                        let reg = self.get_register_pair(source);
                        self.set_register_pair(source, reg.wrapping_add(1));
                        bus.read_ram(reg)
                    }
                    OpMemAddress16::RegisterDecrease(source) => {
                        let reg = self.get_register_pair(source);
                        self.set_register_pair(source, reg.wrapping_sub(1));
                        bus.read_ram(reg)
                    }
                    OpMemAddress16::Immediate => {
                        let lsb = self.read_immediate(bus) as u16;
                        let msb = self.read_immediate(bus) as u16;
                        bus.read_ram((msb << 8) | lsb)
                    }
                };

                self.set_register(target, val);
            }
            Opcode::LdMemR(target, source) => {
                let addr = match target {
                    OpMemAddress16::Register(target) => {
                        self.get_register_pair(target)
                    }
                    OpMemAddress16::RegisterIncrease(target) => {
                        let reg = self.get_register_pair(target);
                        self.set_register_pair(target, reg.wrapping_add(1));
                        reg
                    }
                    OpMemAddress16::RegisterDecrease(target) => {
                        let reg = self.get_register_pair(target);
                        self.set_register_pair(target, reg.wrapping_sub(1));
                        reg
                    }
                    OpMemAddress16::Immediate => {
                        let lsb = self.read_immediate(bus) as u16;
                        let msb = self.read_immediate(bus) as u16;
                        (msb << 8) | lsb
                    }
                };

                bus.write_ram(addr, self.get_register(source));
            }
            Opcode::LdMemImm(target) => {
                let immediate = self.read_immediate(bus);
                bus.write_ram(self.get_register_pair(target), immediate);
            }
            Opcode::LdhRead(target, source) => {
                let addr = 0xFF00 | match source {
                    OpMemAddress8::Register(source) => self.get_register(source),
                    OpMemAddress8::Immediate => self.read_immediate(bus),
                } as u16;

                self.set_register(target, bus.read_ram(addr));
            }
            Opcode::LdhWrite(target, source) => {
                let addr = 0xFF00 | match target {
                    OpMemAddress8::Register(target) => self.get_register(target),
                    OpMemAddress8::Immediate => self.read_immediate(bus),
                } as u16;

                bus.write_ram(addr, self.get_register(source));
            }
            Opcode::Ld16RImm(target) => {
                let lsb = self.read_immediate(bus) as u16;
                let msb = self.read_immediate(bus) as u16;

                // TODO: This affects SP, not AF. Should add SP to enum and add custom try_from
                self.set_register_pair(target, (msb << 8) | lsb);
            }
            Opcode::Ld16MemSp => {
                let lsb = self.read_immediate(bus) as u16;
                let msb = self.read_immediate(bus) as u16;
                let addr = (msb << 8) | lsb;
                bus.write_ram(addr, (self.sp & 0x00FF) as u8);
                bus.write_ram(addr + 1, ((self.sp & 0xFF00) >> 8) as u8);

            }
            Opcode::Ld16SpHL => {
                self.sp = self.get_register_pair(RegisterPair::HL);
            }
            Opcode::Push(source) => {
                let source = self.get_register_pair(source);
                self.sp = self.sp.wrapping_sub(1);
                bus.write_ram(self.sp, ((source & 0xFF00) >> 8) as u8);
                self.sp = self.sp.wrapping_sub(1);
                bus.write_ram(self.sp, (source & 0x00FF) as u8);
            }
            Opcode::Pop(target) => {
                let lsb = bus.read_ram(self.sp) as u16;
                self.sp = self.sp.wrapping_add(1);
                let msb = bus.read_ram(self.sp) as u16;
                self.sp = self.sp.wrapping_add(1);
                self.set_register_pair(target, (msb << 8) | lsb);
            }
            Opcode::AluR(alu_op, source) => {
                let val = self.get_register(source);
                self.run_alu(alu_op, val);
            }
            Opcode::AluImm(alu_op) => {
                let val = self.read_immediate(bus);
                self.run_alu(alu_op, val);
            }
            Opcode::AluMem(alu_op) => {
                let val = bus.read_ram(self.get_register_pair(RegisterPair::HL));
                self.run_alu(alu_op, val);
            }
            Opcode::IncR(source) => {
                let val = self.get_register(source);
                let result = val.wrapping_add(1);

                self.f.set(FlagRegister::H, (val & 0x0F) + 1 > 0x0F);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                self.set_register(source, result);
            }
            Opcode::IncMem => {
                let addr = self.get_register_pair(RegisterPair::HL);
                let val = bus.read_ram(addr);
                let result = val.wrapping_add(1);

                self.f.set(FlagRegister::H, (val & 0x0F) + 1 > 0x0F);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                bus.write_ram(addr, result);
            }
            Opcode::DecR(source) => {
                let val = self.get_register(source);
                let result = val.wrapping_sub(1);

                self.f.set(FlagRegister::H, (val & 0x0F) == 0);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                self.set_register(source, result);
            }
            Opcode::DecMem => {
                let addr = self.get_register_pair(RegisterPair::HL);
                let val = bus.read_ram(addr);
                let result = val.wrapping_sub(1);

                self.f.set(FlagRegister::H, (val & 0x0F) == 0);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                bus.write_ram(addr, result);
            }
            Opcode::Daa => {
                let mut adjustment = if self.f.contains(FlagRegister::C)
                    || (!self.f.contains(FlagRegister::N) && self.a > 0x99)
                {
                    0x60
                } else {
                    0
                };

                if self.f.contains(FlagRegister::H)
                    || (!self.f.contains(FlagRegister::N) && (self.a & 0x0F) > 0x09)
                {
                    adjustment |= 0x06;
                }

                self.a = self.a.wrapping_sub(adjustment);
                self.f.set(FlagRegister::C, adjustment >= 0x60);
                self.f.set(FlagRegister::H, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Opcode::Cpl => {
                self.a = self.a ^ 0xFF;
                self.f.set(FlagRegister::H, true);
                self.f.set(FlagRegister::N, true);
            }
            Opcode::Add16HL(source) => {
                let val = self.get_register_pair(RegisterPair::HL);
                let source = self.get_register_pair(source);
                let (result, carry) = val.overflowing_add(source);
                let half_carry = (val & 0x07FF) + (source & 0x07FF) > 0x07FF;

                self.set_register_pair(RegisterPair::HL, result);
                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
            }
            Opcode::Add16SPSigned => {
                // Reinterpret the immediate as signed, then convert to unsigned u16 equivalent
                let immediate = self.read_immediate(bus) as i8 as i16 as u16;
                let carry = (self.sp & 0x00FF) + (immediate & 0x00FF) > 0x00FF;
                let half_carry = (self.sp & 0x000F) + (immediate & 0x000F) > 0x000F;

                self.sp = self.sp.wrapping_add(immediate);
                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, false);
            }
            Opcode::Inc16R(source) => {
                self.set_register_pair(source, self.get_register_pair(source).wrapping_add(1));
            }
            Opcode::Dec16R(source) => {
                self.set_register_pair(source, self.get_register_pair(source).wrapping_sub(1));
            }
            Opcode::Ld16HLSPSigned => {
                // Reinterpret the immediate as signed, then convert to unsigned u16 equivalent
                let immediate = self.read_immediate(bus) as i8 as i16 as u16;
                let carry = (self.sp & 0x00FF) + (immediate & 0x00FF) > 0x00FF;
                let half_carry = (self.sp & 0x000F) + (immediate & 0x000F) > 0x000F;

                self.set_register_pair(RegisterPair::HL,self.sp.wrapping_add(immediate));
                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, false);
            }
        }
    }

    fn read_immediate(&mut self, bus: &mut CpuBus) -> u8 {
        let immediate = bus.read_ram(self.pc);
        self.pc = self.pc.wrapping_add(1);
        immediate
    }

    fn run_alu(&mut self, alu_op: Alu, val: u8) {
        match alu_op {
            Alu::Add => {
                let (result, carry) = self.a.overflowing_add(val);
                let half_carry = (self.a & 0x0F) + (val & 0x0F) > 0x0F;

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::Adc => {
                let carry_flag = self.f.contains(FlagRegister::C) as u8;

                // Would use carrying_add if it was in stable
                let (r1, c1) = self.a.overflowing_add(val);
                let (r2, c2) = r1.overflowing_add(carry_flag);
                let (result, carry) = (r2, c1 | c2);
                let half_carry = (self.a & 0x0F) + (val & 0x0F) + carry_flag > 0x0F;

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::Sub => {
                let (result, carry) = self.a.overflowing_sub(val);
                let half_carry = (self.a & 0x0F) < (val & 0x0F);

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::Sbc => {
                let carry_flag = self.f.contains(FlagRegister::C) as u8;

                // Would use carrying_sub if it was in stable
                let (r1, c1) = self.a.overflowing_sub(val);
                let (r2, c2) = r1.overflowing_sub(carry_flag);
                let (result, carry) = (r2, c1 | c2);
                let half_carry = (self.a & 0x0F) < (val & 0x0F) + carry_flag;

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::And => {
                self.a &= val;
                self.f.set(FlagRegister::C, false);
                self.f.set(FlagRegister::H, true);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Alu::Xor => {
                self.a ^= val;
                self.f.set(FlagRegister::C, false);
                self.f.set(FlagRegister::H, false);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Alu::Or => {
                self.a |= val;
                self.f.set(FlagRegister::C, false);
                self.f.set(FlagRegister::H, false);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Alu::Cp => {
                let (result, carry) = self.a.overflowing_sub(val);
                let half_carry = (self.a & 0x0F) < (val & 0x0F);

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
            }
        }
    }

    fn get_register(&self, reg: Register) -> u8 {
        match reg {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        }
    }

    fn set_register(&mut self, reg: Register, val: u8) {
        match reg {
            Register::B => self.b = val,
            Register::C => self.c = val,
            Register::D => self.d = val,
            Register::E => self.e = val,
            Register::H => self.h = val,
            Register::L => self.l = val,
            Register::A => self.a = val,
        }
    }

    fn get_register_pair(&self, reg: RegisterPair) -> u16 {
        match reg {
            RegisterPair::BC => ((self.b as u16) << 8) | (self.c as u16),
            RegisterPair::DE => ((self.d as u16) << 8) | (self.e as u16),
            RegisterPair::HL => ((self.h as u16) << 8) | (self.l as u16),
            RegisterPair::SP => self.sp,
            RegisterPair::AF => ((self.a as u16) << 8) | ((self.f.bits & 0xF0) as u16),
        }
    }

    fn set_register_pair(&mut self, reg: RegisterPair, val: u16) {
        match reg {
            RegisterPair::BC => {
                self.b = ((val & 0xFF00) >> 8) as u8;
                self.c = (val & 0x00FF) as u8
            }
            RegisterPair::DE => {
                self.d = ((val & 0xFF00) >> 8) as u8;
                self.e = (val & 0x00FF) as u8
            }
            RegisterPair::HL => {
                self.h = ((val & 0xFF00) >> 8) as u8;
                self.l = (val & 0x00FF) as u8
            }
            RegisterPair::SP => {
                self.sp = val;
            }
            RegisterPair::AF => {
                self.a = ((val & 0xFF00) >> 8) as u8;
                self.f.bits = (val & 0x00F0) as u8
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cartridge;
    use crate::Ppu;
    use crate::RomParserError;
    use crate::WRAM_BANK_SIZE;
    use alloc::vec;

    struct MockEmulator {
        pub cartridge: Cartridge,
        pub cpu: Cpu,
        pub wram: [u8; WRAM_BANK_SIZE as usize * 4],
        pub ppu: Ppu,
    }

    impl MockEmulator {
        pub fn new() -> Result<Self, RomParserError> {
            let mut rom = vec![0; 0x200];
            rom[0x14d] = 231;
            let cartridge = Cartridge::load(&rom, None)?;

            let emulator = Self {
                cartridge,
                cpu: Default::default(),
                wram: [0u8; WRAM_BANK_SIZE as usize * 4],
                ppu: Default::default(),
            };

            Ok(emulator)
        }
    }

    /// Executes `n` instructions and returns
    fn execute_n(emu: &mut MockEmulator, n: usize) {
        let mut bus = borrow_cpu_bus!(emu);
        for _ in 0..n {
            loop {
                // Because of the fetch-execute overlap, running the last cycle fetches the next
                // instruction. We need to run and break in this case to go to the next n
                if emu.cpu.cycles == 1 {
                    emu.cpu.clock(&mut bus);
                    break;
                } else {
                    emu.cpu.clock(&mut bus);
                }
            }
        }
    }

    #[test]
    fn test_ld_rr() {
        let mut emu = MockEmulator::new().unwrap();

        // TODO: Convert this to rom when the bus actually use the cartridge and not wram
        emu.wram[0] = 0x40; // B,B
        emu.wram[1] = 0x41; // B,C
        emu.wram[2] = 0x42; // B,D
        emu.wram[3] = 0x43; // B,E
        emu.wram[4] = 0x44; // B,H
        emu.wram[5] = 0x45; // B,L
        emu.wram[6] = 0x47; // B,A
        emu.wram[7] = 0x78; // A,B
        emu.wram[8] = 0x60; // H,B
        emu.wram[9] = 0x6A; // L,D

        emu.cpu.b = 1;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 1);

        emu.cpu.c = 2;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 2);

        emu.cpu.d = 3;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 3);

        emu.cpu.e = 4;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 4);

        emu.cpu.h = 5;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 5);

        emu.cpu.l = 6;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 6);

        emu.cpu.a = 7;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 7);

        emu.cpu.b = 20;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.a, 20);

        emu.cpu.b = 21;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.h, 21);

        emu.cpu.d = 30;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.l, 30);
    }

    #[test]
    fn test_ld_r_imm() {
        let mut emu = MockEmulator::new().unwrap();

        // TODO: Convert this to rom when the bus actually use the cartridge and not wram
        emu.wram[0] = 0x06; // B,n
        emu.wram[1] = 1;
        emu.wram[2] = 0x3E; // A,n
        emu.wram[3] = 255;

        execute_n(&mut emu, 2);
        assert_eq!(emu.cpu.b, 1);
        assert_eq!(emu.cpu.a, 255);
    }

    #[test]
    fn test_ld16_r_imm() {
        let mut emu = MockEmulator::new().unwrap();

        // TODO: Convert this to rom when the bus actually use the cartridge and not wram
        emu.wram[0] = 0x01; // BC,nn
        emu.wram[1] = 0x10; // lsb
        emu.wram[2] = 0x20; // msb
        emu.wram[3] = 0x11; // DE,nn
        emu.wram[4] = 0x30; // lsb
        emu.wram[5] = 0x40; // msb
        emu.wram[6] = 0x21; // HL,nn
        emu.wram[7] = 0x50; // lsb
        emu.wram[8] = 0x60; // msb
        emu.wram[9] = 0x31; // SP,nn
        emu.wram[10] = 0x70; // lsb
        emu.wram[11] = 0x80; // msb

        execute_n(&mut emu, 4);
        assert_eq!(emu.cpu.b, 0x20);
        assert_eq!(emu.cpu.c, 0x10);
        assert_eq!(emu.cpu.d, 0x40);
        assert_eq!(emu.cpu.e, 0x30);
        assert_eq!(emu.cpu.h, 0x60);
        assert_eq!(emu.cpu.l, 0x50);
        assert_eq!(emu.cpu.sp, 0x8070);
    }

    #[test]
    fn test_push() {
        let mut emu = MockEmulator::new().unwrap();

        // TODO: Convert this to rom when the bus actually use the cartridge and not wram
        emu.wram[0] = 0xC5; // BC

        emu.cpu.sp = 0x500;
        emu.cpu.b = 0x10;
        emu.cpu.c = 0x20;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.sp, 0x4FE);
        assert_eq!(emu.wram[0x4FF], 0x10);
        assert_eq!(emu.wram[0x4FE], 0x20);
    }

    #[test]
    fn test_pop() {
        let mut emu = MockEmulator::new().unwrap();

        // TODO: Convert this to rom when the bus actually use the cartridge and not wram
        emu.wram[0] = 0xC1; // BC
        emu.wram[0x4FE] = 0x20;
        emu.wram[0x4FF] = 0x10;

        emu.cpu.sp = 0x4FE;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.sp, 0x500);
        assert_eq!(emu.cpu.b, 0x10);
        assert_eq!(emu.cpu.c, 0x20);
    }
}

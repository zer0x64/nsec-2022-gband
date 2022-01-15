mod decoder;

use bitflags::bitflags;

use crate::bus::CpuBus;
use decoder::{Opcode, Register, RegisterPair};

bitflags! {
    pub struct FlagRegister: u8 {
        const UNUSED = 0x0F;
        const C = 0x10;
        const H = 0x20;
        const N = 0x40;
        const Z = 0x80;
    }
}

/*#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
enum Alu {
    AddA = 0,
    AdcA = 1,
    Sub = 2,
    SbcA = 3,
    And = 4,
    Xor = 5,
    Or = 6,
    Cp = 7
}

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
enum Rot {
    Rlc = 0,
    Rrc = 1,
    Rl = 2,
    Rr = 3,
    Sla = 4,
    Sra = 5,
    Swap = 6,
    Srl = 7
}*/

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

    pub fn fetch(&mut self, bus: &mut CpuBus) {
        self.opcode_latch = Opcode::from_u8(bus.read_ram(self.pc));
        self.pc = self.pc.wrapping_add(1);

        self.cycles = self.opcode_latch.cycles();
    }

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
                let immediate = bus.read_ram(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.set_register(target, immediate);
            }
            Opcode::LdRMem(target, source) => {
                let val = bus.read_ram(self.get_register_pair(source));
                self.set_register(target, val);
            }
            Opcode::LdMemR(target, source) => {
                bus.write_ram(self.get_register_pair(target), self.get_register(source));
            }
            Opcode::LdMemImm(target) => {
                let immediate = bus.read_ram(self.pc);
                self.pc = self.pc.wrapping_add(1);
                bus.write_ram(self.get_register_pair(target), immediate);
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
            RegisterPair::AF => {
                self.a = ((val & 0xFF00) >> 8) as u8;
                self.f.bits = (val & 0x00F0) as u8
            }
        }
    }
}

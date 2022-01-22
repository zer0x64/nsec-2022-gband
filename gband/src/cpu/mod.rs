mod decoder;

use bitflags::bitflags;

use crate::bus::CpuBus;
use decoder::{Opcode, OpMemAddress16, Register, RegisterPair};
use crate::cpu::decoder::OpMemAddress8;

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
        }
    }

    fn read_immediate(&mut self, bus: &mut CpuBus) -> u8 {
        let immediate = bus.read_ram(self.pc);
        self.pc = self.pc.wrapping_add(1);
        immediate
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
}

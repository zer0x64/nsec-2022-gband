#![no_std]

extern crate alloc;

#[macro_use]
pub mod bus; // TODO: Revert pub added for criterion

mod cartridge;
mod joypad_state;
mod cpu;
mod ppu;
mod rgb_palette;
pub mod utils;

pub use cartridge::RomParserError;
pub use joypad_state::JoypadState;
pub use cpu::Cpu;
pub use ppu::{Frame, Ppu, FRAME_HEIGHT, FRAME_WIDTH};

// TODO: Revert pub added for criterion
pub use cartridge::Cartridge;

const WRAM_BANK_SIZE: u16 = 0x1000; // 4KiB

pub struct Emulator {
    // == Cartridge Related Hardware== //
    cartridge: Cartridge,

    // == CPU Related Hardware == //
    cpu: Cpu,
    wram: [u8; WRAM_BANK_SIZE as usize * 8],
    // 0x7F instead of 0x80 is not a mistake, as the last byte is used to access interupts
    hram: [u8; 0x7F],

    // == PPU Related Hardware == //
    ppu: Ppu,

    // == IP Related Hardware == //

    // == IO Hardware ==
    joypad_state: JoypadState,
    joypad_register: u8,

    // == Emulation Specific Data == //
    serial_port_buffer: alloc::vec::Vec<u8>,
    clock_count: u8,
}

impl Emulator {
    pub fn new(rom: &[u8], save_data: Option<&[u8]>) -> Result<Self, RomParserError> {
        let cartridge = Cartridge::load(rom, save_data)?;

        let emulator = Self {
            cartridge,
            cpu: Default::default(),
            wram: [0u8; WRAM_BANK_SIZE as usize * 8],
            hram: [0u8; 0x7F],

            ppu: Default::default(),

            joypad_state: Default::default(),
            joypad_register: Default::default(),

            serial_port_buffer: alloc::vec::Vec::with_capacity(256),
            clock_count: 0,
        };

        Ok(emulator)
    }

    pub fn clock(&mut self) -> Option<Frame> {
        self.clock_count += 1;

        // clock_count is at ~4MHz
        // PPU is clocked at ~4MHz
        self.ppu.clock();

        // We clock CPU on M-cycles, at ~1MHz on regular mode and ~2MHz on CGB double speed mode
        // This means we clock it every 2 or 4 cycles
        if self.clock_count == 4 {
            let mut cpu_bus = borrow_cpu_bus!(self);
            self.cpu.clock(&mut cpu_bus);

            if self.clock_count == 4 {
                self.clock_count = 0;
            }
        };

        self.ppu.ready_frame()
    }

    pub fn set_joypad(&mut self, state: JoypadState) {
        self.joypad_state = state
    }

    pub fn get_save_data(&self) -> Option<&[u8]> {
        self.cartridge.get_save_data()
    }

    #[cfg(feature = "debugger")]
    pub fn disassemble(
        &self,
        _start: u16,
        _end: u16,
    ) -> alloc::vec::Vec<(Option<u8>, u16, alloc::string::String)> {
        // TODO
        alloc::vec::Vec::new()
    }

    #[cfg(feature = "debugger")]
    pub fn mem_dump(&mut self, start: u16, end: u16) -> alloc::vec::Vec<u8> {
        let mut data = alloc::vec::Vec::new();

        // TODO
        /*for addr in start..=end {
            let mut bus = borrow_cpu_bus!(self);
            data.push(self.cpu.mem_dump(&mut bus, addr));
        }*/

        data
    }

    #[cfg(feature = "debugger")]
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}

#[test]
fn test() {
    let mut rom = [0u8; 0x150];
    rom[0x14d] = 231;
    let mut emu = Emulator::new(&rom, None).unwrap();

    for _ in 0..10 {
        emu.clock();
    }
}

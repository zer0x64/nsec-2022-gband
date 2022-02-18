use super::Mapper;
use crate::cartridge::CartridgeReadTarget;

pub struct Mbc5 {
    bank_mask: usize,
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
}

impl Mbc5 {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Mbc5 {
    fn default() -> Self {
        Self {
            bank_mask: 0xF,
            ram_enable: false,
            rom_bank_number: 0x01,
            ram_bank_number: 0x00,
        }
    }
}

impl Mapper for Mbc5 {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget {
        match addr {
            0x0000..=0x3FFF => {
                // First bank
                // Fixed to bank 0
                let mask = 0x3FFF;
                CartridgeReadTarget::Rom((addr & mask) as usize)
            }
            0x4000..=0x7FFF => {
                // Switchable ROM banks
                let mask = 0x3fff;
                let addr = (addr & mask) as usize;

                let mut bank = (self.rom_bank_number as usize) << 14usize;
                CartridgeReadTarget::Rom(bank | addr)
            }
            0xA000..=0xBFFF => {
                // RAM range.
                // Can only be used when enabled
                if self.ram_enable {
                    let mask = 0x1fff;
                    let addr = (addr & mask) as usize;

                    let mut bank = (self.ram_bank_number as usize) << 13usize;
                    CartridgeReadTarget::Ram(bank | addr)
                } else {
                    CartridgeReadTarget::Error
                }
            }
            _ => {
                log::warn!("Read on cartridge at {addr}, which isn't supposed to be mapped to the cartridge");
                CartridgeReadTarget::Error
            }
        }
    }

    fn map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match addr {
            0x0000..=0x1FFF => {
                // Enables or diables the RAM
                self.ram_enable = data & 0xf == 0x0A;
                None
            }
            0x2000..=0x2FFF => {
                // Set ROM Bank Number
                // Used to bank switch range 0x4000 - 0x7FFF
                let bank_number = data & (self.bank_mask as u8) & 0x1F;

                if bank_number == 0 {
                    // This register cannot be 0 and default to 1 if we try to set it to 0
                    self.rom_bank_number = 1;
                } else {
                    self.rom_bank_number = bank_number;
                }
                None
            }
            0x3000..=0x3FFF => {

            }
            0x4000..=0x5FFF => {
                // Two additionnal bits used for bank switching on cartridge with large ROM or RAM
                self.ram_bank_number_or_upper_rom_bank = data & 0b11;
                None
            }
            _ => {
                log::warn!("Write on cartridge at {addr}, which isn't supposed to be mapped to the cartridge");
                None
            }
        }
    }
}

use bitflags::bitflags;

#[derive(Clone, Copy)]
pub struct InterruptState {
    pub enable: InterruptReg,
    pub status: InterruptReg,
}

impl Default for InterruptState {
    fn default() -> Self {
        Self {
            enable: Default::default(),
            status: InterruptReg::from_bits_truncate(0xE1),
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct InterruptReg: u8 {
        const VBLANK = 0x01;
        const LCD_STAT = 0x02;
        const TIMER = 0x04;
        const SERIAL = 0x08;
        const JOYPAD = 0x10;
        const UNUSED = 0xE0;
    }
}

#[derive(Clone, Copy)]
pub enum FifoMode {
    HBlank,
    VBlank,
    OamScan(OamScanState),
    Drawing,
}

#[derive(Clone, Copy, Default)]
pub struct OamScanState {
    pub oam_pointer: usize,
    pub secondary_oam_pointer: usize,
    pub is_visible: bool,
}

impl Default for FifoMode {
    fn default() -> Self {
        Self::OamScan(Default::default())
    }
}

impl From<FifoMode> for u8 {
    fn from(item: FifoMode) -> u8 {
        match item {
            FifoMode::HBlank => 0,
            FifoMode::VBlank => 1,
            FifoMode::OamScan(_) => 2,
            FifoMode::Drawing => 3,
        }
    }
}

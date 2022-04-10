#[derive(Clone, Copy)]
pub enum FifoMode {
    HBlank,
    VBlank,
    OamScan(OamScanState),
    Drawing(DrawingState),
}

#[derive(Clone, Copy, Default)]
pub struct OamScanState {
    pub oam_pointer: usize,
    pub secondary_oam_pointer: usize,
    pub is_visible: bool,
}

#[derive(Clone, Copy, Default)]
pub struct DrawingState {
    pub pixel_fetcher: PixelFetcherState,
    pub cycle: u8,

    pub fetcher_x: u8,
    pub is_window: bool,

    pub is_sprite: bool,
    pub sprite_idx: u8,

    pub tile_idx: u8,
    pub buffer: [u16; 8],
}

impl DrawingState {
    pub fn reset(&mut self) {
        self.pixel_fetcher = Default::default();
        self.cycle = 0;
        self.tile_idx = 0;
        self.buffer = Default::default();
    }
}

#[derive(Clone, Copy)]
pub enum PixelFetcherState {
    GetTile,
    GetTileLow,
    GetTileHigh,
    Push,
}

impl Default for PixelFetcherState {
    fn default() -> Self {
        Self::GetTile
    }
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
            FifoMode::Drawing(_) => 3,
        }
    }
}

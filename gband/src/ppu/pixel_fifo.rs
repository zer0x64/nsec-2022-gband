#[derive(Default, Clone)]
pub struct PixelFifo {
    pub fifo: u128,
    pub n_pixels: u8,
}

impl PixelFifo {
    pub fn is_empty(&self) -> bool {
        self.n_pixels == 0
    }

    pub fn pop(&mut self) -> Option<u16> {
        if self.n_pixels == 0 {
            None
        } else {
            let res = self.fifo & 0xf0000000000000000000000000000 >> 112;
            self.fifo = self.fifo.overflowing_shl(16).0;
            self.n_pixels -= 1;

            Some(res as u16)
        }
    }

    pub fn load(&mut self, value: u128) {
        self.fifo = value;
        self.n_pixels = 8;
    }
}

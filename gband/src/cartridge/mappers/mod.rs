use super::CartridgeReadTarget;

mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod no_mapper;

pub use mbc1::Mbc1;
pub use mbc2::Mbc2;
pub use mbc3::Mbc3;
pub use mbc5::Mbc5;
pub use no_mapper::NoMapper;

pub trait Mapper: Send + Sync {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget;
    fn map_write(&mut self, addr: u16, data: u8) -> Option<usize>;
}

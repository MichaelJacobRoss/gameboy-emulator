pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub mod bus;
pub mod flat_memory;
pub use bus::Bus;
pub use flat_memory::FlatMemory;

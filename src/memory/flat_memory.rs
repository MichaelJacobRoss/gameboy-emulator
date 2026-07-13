use super::MemoryBus;

pub struct FlatMemory {
    data: [u8; 0x10000],
}

impl FlatMemory {
    pub fn new() -> Self {
        Self { data: [0; 0x10000] }
    }
}

impl Default for FlatMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBus for FlatMemory {
    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}

use std::io::Write;

use super::MemoryBus;

pub struct Bus {
    rom: Vec<u8>,
    vram: [u8; 0x2000],
    eram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    hram: [u8; 0x7F],
    pub ie: u8,
    pub if_reg: u8,
    sb: u8,
    sc: u8,
}

enum Addr {
    Rom(usize),
    Vram(usize),
    Eram(usize),
    Wram(usize),
    Oam(usize),
    Hram(usize),
    Ie,
    IfReg,
    Sb,
    Sc,
    Unusable,
    Unmapped,
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            vram: [0; 0x2000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            hram: [0; 0x7F],
            ie: 0,
            if_reg: 0,
            sb: 0,
            sc: 0,
        }
    }

    fn resolve(addr: u16) -> Addr {
        match addr {
            0x0000..=0x7FFF => Addr::Rom(addr as usize),
            0x8000..=0x9FFF => Addr::Vram((addr - 0x8000) as usize),
            0xA000..=0xBFFF => Addr::Eram((addr - 0xA000) as usize),
            0xC000..=0xDFFF => Addr::Wram((addr - 0xC000) as usize),
            0xE000..=0xFDFF => Addr::Wram((addr - 0xE000) as usize),
            0xFE00..=0xFE9F => Addr::Oam((addr - 0xFE00) as usize),
            0xFEA0..=0xFEFF => Addr::Unusable,
            0xFF01 => Addr::Sb,
            0xFF02 => Addr::Sc,
            0xFF0F => Addr::IfReg,
            0xFF80..=0xFFFE => Addr::Hram((addr - 0xFF80) as usize),
            0xFFFF => Addr::Ie,
            _ => Addr::Unmapped,
        }
    }
}

impl MemoryBus for Bus {
    fn read(&self, addr: u16) -> u8 {
        match Bus::resolve(addr) {
            Addr::Rom(i) => self.rom.get(i).copied().unwrap_or(0xFF),
            Addr::Vram(i) => self.vram[i],
            Addr::Eram(i) => self.eram[i],
            Addr::Wram(i) => self.wram[i],
            Addr::Oam(i) => self.oam[i],
            Addr::Hram(i) => self.hram[i],
            Addr::Ie => self.ie,
            Addr::IfReg => self.if_reg,
            Addr::Sb => self.sb,
            Addr::Sc => self.sc,
            Addr::Unusable | Addr::Unmapped => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match Bus::resolve(addr) {
            Addr::Rom(_) | Addr::Unusable | Addr::Unmapped => {}
            Addr::Vram(i) => self.vram[i] = value,
            Addr::Eram(i) => self.eram[i] = value,
            Addr::Wram(i) => self.wram[i] = value,
            Addr::Oam(i) => self.oam[i] = value,
            Addr::Hram(i) => self.hram[i] = value,
            Addr::Ie => self.ie = value,
            Addr::IfReg => self.if_reg = value,
            Addr::Sb => self.sb = value,
            Addr::Sc => {
                self.sc = value;
                if value & 0x80 != 0 {
                    print!("{}", self.sb as char);
                    let _ = std::io::stdout().flush();
                    self.sc &= !0x80;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bus() -> Bus {
        Bus::new(vec![0u8; 0x8000])
    }

    #[test]
    fn rom_read_returns_correct_byte() {
        let mut rom = vec![0u8; 0x8000];
        rom[0x0100] = 0xAB;
        let bus = Bus::new(rom);
        assert_eq!(bus.read(0x0100), 0xAB);
    }

    #[test]
    fn rom_write_is_ignored() {
        let bus_before = make_bus();
        let mut bus = make_bus();
        bus.write(0x0100, 0xFF);
        assert_eq!(bus.read(0x0100), bus_before.read(0x0100));
    }

    #[test]
    fn rom_read_past_end_returns_0xff() {
        let bus = Bus::new(vec![0u8; 0x100]);
        assert_eq!(bus.read(0x0200), 0xFF);
    }

    #[test]
    fn vram_read_write() {
        let mut bus = make_bus();
        bus.write(0x8000, 0x12);
        bus.write(0x9FFF, 0x34);
        assert_eq!(bus.read(0x8000), 0x12);
        assert_eq!(bus.read(0x9FFF), 0x34);
    }

    #[test]
    fn eram_read_write() {
        let mut bus = make_bus();
        bus.write(0xA000, 0x56);
        bus.write(0xBFFF, 0x78);
        assert_eq!(bus.read(0xA000), 0x56);
        assert_eq!(bus.read(0xBFFF), 0x78);
    }

    #[test]
    fn wram_read_write() {
        let mut bus = make_bus();
        bus.write(0xC000, 0x9A);
        bus.write(0xDFFF, 0xBC);
        assert_eq!(bus.read(0xC000), 0x9A);
        assert_eq!(bus.read(0xDFFF), 0xBC);
    }

    #[test]
    fn echo_ram_reads_wram() {
        let mut bus = make_bus();
        bus.write(0xC123, 0x42);
        assert_eq!(bus.read(0xE123), 0x42);
    }

    #[test]
    fn echo_ram_writes_wram() {
        let mut bus = make_bus();
        bus.write(0xE123, 0x42);
        assert_eq!(bus.read(0xC123), 0x42);
    }

    #[test]
    fn oam_read_write() {
        let mut bus = make_bus();
        bus.write(0xFE00, 0xDE);
        bus.write(0xFE9F, 0xAD);
        assert_eq!(bus.read(0xFE00), 0xDE);
        assert_eq!(bus.read(0xFE9F), 0xAD);
    }

    #[test]
    fn unusable_read_returns_0xff() {
        let bus = make_bus();
        assert_eq!(bus.read(0xFEA0), 0xFF);
        assert_eq!(bus.read(0xFEFF), 0xFF);
    }

    #[test]
    fn unusable_write_is_ignored() {
        let mut bus = make_bus();
        bus.write(0xFEA0, 0x55);
        assert_eq!(bus.read(0xFEA0), 0xFF);
    }

    #[test]
    fn hram_read_write() {
        let mut bus = make_bus();
        bus.write(0xFF80, 0x11);
        bus.write(0xFFFE, 0x22);
        assert_eq!(bus.read(0xFF80), 0x11);
        assert_eq!(bus.read(0xFFFE), 0x22);
    }

    #[test]
    fn ie_read_write() {
        let mut bus = make_bus();
        bus.write(0xFFFF, 0x1F);
        assert_eq!(bus.read(0xFFFF), 0x1F);
    }

    #[test]
    fn if_reg_read_write() {
        let mut bus = make_bus();
        bus.write(0xFF0F, 0x05);
        assert_eq!(bus.read(0xFF0F), 0x05);
    }

    #[test]
    fn serial_sb_read_write() {
        let mut bus = make_bus();
        bus.write(0xFF01, 0x41);
        assert_eq!(bus.read(0xFF01), 0x41);
    }

    #[test]
    fn serial_sc_transfer_clears_bit7() {
        let mut bus = make_bus();
        bus.write(0xFF01, 0x41);
        bus.write(0xFF02, 0x81);
        assert_eq!(
            bus.read(0xFF02) & 0x80,
            0,
            "SC bit 7 should be cleared after transfer"
        );
    }

    #[test]
    fn serial_sc_without_bit7_does_not_trigger_transfer() {
        let mut bus = make_bus();
        bus.write(0xFF01, 0x41);
        bus.write(0xFF02, 0x01);
        assert_eq!(bus.read(0xFF02), 0x01);
    }

    #[test]
    fn serial_sc_preserves_lower_bits_after_transfer() {
        let mut bus = make_bus();
        bus.write(0xFF02, 0x81);
        assert_eq!(bus.read(0xFF02), 0x01);
    }

    #[test]
    fn unmapped_io_read_returns_0xff() {
        let bus = make_bus();
        assert_eq!(bus.read(0xFF50), 0xFF);
    }

    #[test]
    fn unmapped_io_write_is_ignored() {
        let mut bus = make_bus();
        bus.write(0xFF50, 0x01);
        assert_eq!(bus.read(0xFF50), 0xFF);
    }
}

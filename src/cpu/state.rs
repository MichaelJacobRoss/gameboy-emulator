use super::Cpu;
use crate::memory::MemoryBus;

/// Full register state plus IME. Memory is not included; RAM contents belong
/// to the `MemoryBus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuState {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub ime: bool,
}

impl From<&Cpu> for CpuState {
    fn from(cpu: &Cpu) -> Self {
        Self {
            pc: cpu.pc,
            sp: cpu.sp,
            a: cpu.a,
            f: cpu.f,
            b: cpu.b,
            c: cpu.c,
            d: cpu.d,
            e: cpu.e,
            h: cpu.h,
            l: cpu.l,
            ime: cpu.ime,
        }
    }
}

impl Cpu {
    /// Constructs a `Cpu` at an arbitrary state. F's low nibble is masked,
    /// matching `set_register8`.
    pub fn with_state(state: CpuState, memory: Box<dyn MemoryBus>) -> Self {
        Self {
            memory,
            pc: state.pc,
            sp: state.sp,
            a: state.a,
            f: state.f & 0b11110000,
            b: state.b,
            c: state.c,
            d: state.d,
            e: state.e,
            h: state.h,
            l: state.l,
            ime: state.ime,
            halted: false,
            pending_ime: false,
        }
    }

    pub fn state(&self) -> CpuState {
        CpuState::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::FlatMemory;

    fn sample_state() -> CpuState {
        CpuState {
            pc: 0x1234,
            sp: 0xFFFE,
            a: 0x12,
            f: 0xB0,
            b: 0x34,
            c: 0x56,
            d: 0x78,
            e: 0x9A,
            h: 0xBC,
            l: 0xDE,
            ime: true,
        }
    }

    #[test]
    fn with_state_state_round_trips() {
        let cpu = Cpu::with_state(sample_state(), Box::new(FlatMemory::new()));
        assert_eq!(cpu.state(), sample_state());
    }

    #[test]
    fn with_state_masks_f_low_nibble() {
        let state = CpuState {
            f: 0xBF,
            ..sample_state()
        };
        let cpu = Cpu::with_state(state, Box::new(FlatMemory::new()));
        assert_eq!(cpu.state().f, 0xB0);
    }

    #[test]
    fn with_state_starts_unhalted() {
        let cpu = Cpu::with_state(sample_state(), Box::new(FlatMemory::new()));
        assert!(!cpu.halted());
    }
}

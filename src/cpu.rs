#[macro_use]
pub mod registers;

pub mod flags;

#[cfg(test)]
pub mod snapshot;

use thiserror::Error;

use crate::{
    instruction::{
        Instruction, InstructionDecodeError, InstructionExecuteError, Opcode, OpcodeDecodeError,
    },
    memory::{FlatMemory, MemoryBus},
    util,
};

pub struct Cpu {
    pc: u16,
    sp: u16,
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    ime: bool,
    halted: bool,
    pending_ime: bool,
    memory: Box<dyn MemoryBus>,
}

#[derive(Debug, Error)]
pub enum CpuStepError {
    #[error("0x{pc:04X}: failed to decode opcode: {source}")]
    Opcode { pc: u16, source: OpcodeDecodeError },

    #[error("0x{pc:04X}: failed to decode instruction: {source}")]
    Decode {
        pc: u16,
        source: InstructionDecodeError,
    },

    #[error("0x{pc:04X}: failed to execute instruction: {source}")]
    Execute {
        pc: u16,
        source: InstructionExecuteError,
    },
}

impl Cpu {
    pub(crate) fn fetch_byte(&mut self) -> u8 {
        let value = self.memory.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    pub(crate) fn fetch_word(&mut self) -> u16 {
        let low_byte = self.fetch_byte();
        let high_byte = self.fetch_byte();
        util::concat_bytes(high_byte, low_byte)
    }

    pub(crate) fn read(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub(crate) fn write(&mut self, address: u16, value: u8) {
        self.memory.write(address, value)
    }

    pub(crate) fn set_pc(&mut self, value: u16) {
        self.pc = value
    }

    pub fn new(memory: Box<dyn MemoryBus>) -> Self {
        Self {
            memory,
            pc: 0,
            sp: 0,
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            ime: false,
            halted: false,
            pending_ime: false,
        }
    }

    // Skips the boot ROM by starting with the post-boot DMG register state.
    pub fn new_dmg(memory: Box<dyn MemoryBus>) -> Self {
        Self {
            memory,
            pc: 0x0100,
            sp: 0xFFFE,
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            ime: false,
            halted: false,
            pending_ime: false,
        }
    }

    pub fn step(&mut self) -> Result<(), CpuStepError> {
        if self.pending_ime {
            self.ime = true;
            self.pending_ime = false;
        }

        if self.halted {
            return Ok(());
        }

        let pc = self.pc;
        let opcode = Opcode::try_from(self.fetch_byte())
            .map_err(|source| CpuStepError::Opcode { pc, source })?;
        let instruction = Instruction::decode(opcode, self)
            .map_err(|source| CpuStepError::Decode { pc, source })?;
        instruction
            .execute(self)
            .map_err(|source| CpuStepError::Execute { pc, source })
    }

    pub(crate) fn disable_ime(&mut self) {
        self.ime = false
    }

    pub(crate) fn set_pending_ime(&mut self) {
        self.pending_ime = true
    }

    pub(crate) fn halt(&mut self) {
        self.halted = true
    }
}

#[cfg(test)]
impl Cpu {
    pub fn new_with_program(program: &[u8]) -> Cpu {
        let mut memory = FlatMemory::new();

        for (i, byte) in program.iter().enumerate() {
            memory.write(0x0100 + i as u16, *byte);
        }

        let mut cpu = Cpu::new(Box::new(memory));

        cpu.pc = 0x0100;
        cpu
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn ime(&self) -> bool {
        self.ime
    }

    pub fn halted(&self) -> bool {
        self.halted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::FlatMemory;

    fn dmg_cpu() -> Cpu {
        Cpu::new_dmg(Box::new(FlatMemory::new()))
    }

    #[test]
    fn new_dmg_pc() {
        assert_eq!(dmg_cpu().pc, 0x0100);
    }

    #[test]
    fn new_dmg_sp() {
        assert_eq!(dmg_cpu().sp, 0xFFFE);
    }

    #[test]
    fn new_dmg_af() {
        let cpu = dmg_cpu();
        assert_eq!(cpu.a, 0x01);
        assert_eq!(cpu.f, 0xB0);
    }

    #[test]
    fn new_dmg_bc() {
        let cpu = dmg_cpu();
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.c, 0x13);
    }

    #[test]
    fn new_dmg_de() {
        let cpu = dmg_cpu();
        assert_eq!(cpu.d, 0x00);
        assert_eq!(cpu.e, 0xD8);
    }

    #[test]
    fn new_dmg_hl() {
        let cpu = dmg_cpu();
        assert_eq!(cpu.h, 0x01);
        assert_eq!(cpu.l, 0x4D);
    }

    #[test]
    fn new_dmg_ime_is_false() {
        assert!(!dmg_cpu().ime);
    }
}

use std::convert::TryFrom;
use thiserror::Error;

use crate::{
    cpu::registers::{Register8, Register16},
    reg,
};

pub enum Opcode {
    //Nop
    Nop,

    //Jump
    JpImm16,

    //Ld r8 imm8
    LdR8(Register8),

    // Ld r16 imm8
    LdR16(Register16),

    //Ld r8 r8
    LdR8R8 { dst: Register8, src: Register8 },

    // INC r8
    IncR8(Register8),

    // DEC r8
    DecR8(Register8),

    // INC r16
    IncR16(Register16),

    // DEC r16
    DecR16(Register16),

    // Ld MemHL r8
    LdMemHLR8(Register8),

    LdR8FromMemHL(Register8),

    LdAFromAddr,
    LdAddrA,

    Di,
    Ei,
    Halt,
}

impl TryFrom<u8> for Opcode {
    type Error = OpcodeDecodeError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let invalid = || OpcodeDecodeError::InvalidOpcode(byte);

        // The 8-bit register-indexed blocks all carry their operand in the
        // `0b_ddd` field at bits 3..=5; `(HL)` (index 6) has no `Register8`
        // and so falls through to `InvalidOpcode` until those forms land.
        let dst = || Register8::from_bits(byte >> 3).ok_or_else(invalid);

        let opcode = match byte {
            // NOP
            0x00 => Opcode::Nop,

            // JP a16
            0xC3 => Opcode::JpImm16,

            // LD r8, imm8 (0b00_ddd_110)
            b if b & 0b1100_0111 == 0b0000_0110 => Opcode::LdR8(dst()?),

            // INC r8 (0b00_ddd_100)
            b if b & 0b1100_0111 == 0b0000_0100 => Opcode::IncR8(dst()?),

            // DEC r8 (0b00_ddd_101)
            b if b & 0b1100_0111 == 0b0000_0101 => Opcode::DecR8(dst()?),

            // LD r8, r8 and the (HL) memory variants (0b01_ddd_sss).
            // 0x76 would be `LD (HL), (HL)`; the hardware reuses it as HALT.
            0x40..=0x7F => {
                let dst = Register8::from_bits(byte >> 3);
                let src = Register8::from_bits(byte);
                match (dst, src) {
                    (None, None) => Opcode::Halt,
                    (Some(dst), None) => Opcode::LdR8FromMemHL(dst),
                    (None, Some(src)) => Opcode::LdMemHLR8(src),
                    (Some(dst), Some(src)) => Opcode::LdR8R8 { dst, src },
                }
            }

            // LD A, (a16) / LD (a16), A
            0xFA => Opcode::LdAFromAddr,
            0xEA => Opcode::LdAddrA,

            // LD r16, imm16
            0x01 => Opcode::LdR16(reg!(BC)),
            0x11 => Opcode::LdR16(reg!(DE)),
            0x21 => Opcode::LdR16(reg!(HL)),
            0x31 => Opcode::LdR16(reg!(SP)),

            // INC r16
            0x03 => Opcode::IncR16(reg!(BC)),
            0x13 => Opcode::IncR16(reg!(DE)),
            0x23 => Opcode::IncR16(reg!(HL)),
            0x33 => Opcode::IncR16(reg!(SP)),

            // DEC r16
            0x0B => Opcode::DecR16(reg!(BC)),
            0x1B => Opcode::DecR16(reg!(DE)),
            0x2B => Opcode::DecR16(reg!(HL)),
            0x3B => Opcode::DecR16(reg!(SP)),

            // Interrupt control
            0xF3 => Opcode::Di,
            0xFB => Opcode::Ei,

            _ => return Err(invalid()),
        };

        Ok(opcode)
    }
}

#[derive(Debug, Error)]
pub enum OpcodeDecodeError {
    #[error("Invalid opcode: 0x{0:02X}")]
    InvalidOpcode(u8),
}

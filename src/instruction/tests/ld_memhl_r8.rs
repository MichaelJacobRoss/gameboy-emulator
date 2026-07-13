use crate::{
    cpu::{Cpu, registers::Register8},
    reg,
};

fn test_ld_mem_hl_from_r8(opcode: u8, src: Register8) {
    let addr = 0xC1C1; // Has to be repeated because otherwise H and L will fail
    let value = 0xC1;

    let mut cpu = Cpu::new_with_program(&[opcode]);
    cpu.set_register16(reg!(HL), addr);
    cpu.set_register8(src, value);

    cpu.step().unwrap();

    assert_eq!(
        cpu.read(addr),
        value,
        "Failed to store value from {:?} into memory",
        src
    );
}

#[test]
fn test_ld_mem_hl_from_b() {
    test_ld_mem_hl_from_r8(0x70, reg!(B));
}

#[test]
fn test_ld_mem_hl_from_c() {
    test_ld_mem_hl_from_r8(0x71, reg!(C));
}

#[test]
fn test_ld_mem_hl_from_d() {
    test_ld_mem_hl_from_r8(0x72, reg!(D));
}

#[test]
fn test_ld_mem_hl_from_e() {
    test_ld_mem_hl_from_r8(0x73, reg!(E));
}

#[test]
fn test_ld_mem_hl_from_h() {
    test_ld_mem_hl_from_r8(0x74, reg!(H));
}

#[test]
fn test_ld_mem_hl_from_l() {
    test_ld_mem_hl_from_r8(0x75, reg!(L));
}

#[test]
fn test_ld_mem_hl_from_a() {
    test_ld_mem_hl_from_r8(0x77, reg!(A));
}

//! SingleStepTests/sm83 conformance harness (ROADMAP §1.7).
//!
//! 1000 JSON cases per opcode drive `Cpu` against `FlatMemory` and assert the
//! final register state (including IME) and RAM. Per-cycle bus activity is
//! ignored until Phase 2 cycle counting.
//!
//! The ~500 MB corpus is not vendored. Point the suite at a checkout with
//! `SM83_TEST_DIR`, or clone it to the default location:
//!
//! ```text
//! git clone --depth 1 https://github.com/SingleStepTests/sm83 sm83-data
//! cargo test --features sm83-conformance --test sm83_conformance
//! ```
//!
//! Cases run for every opcode the decoder implements, so adding an opcode to
//! the emulator automatically brings its 1000 cases into the run.

#![cfg(feature = "sm83-conformance")]

use std::path::PathBuf;

use gameboy_emulator::cpu::Cpu;
use gameboy_emulator::cpu::state::CpuState;
use gameboy_emulator::instruction::Opcode;
use gameboy_emulator::memory::{FlatMemory, MemoryBus};
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    initial: State,
    #[serde(rename = "final")]
    end: State,
}

#[derive(Deserialize)]
struct State {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    ime: u8,
    ram: Vec<(u16, u8)>,
}

impl State {
    fn cpu_state(&self) -> CpuState {
        CpuState {
            pc: self.pc,
            sp: self.sp,
            a: self.a,
            f: self.f,
            b: self.b,
            c: self.c,
            d: self.d,
            e: self.e,
            h: self.h,
            l: self.l,
            ime: self.ime != 0,
        }
    }
}

fn test_dir() -> PathBuf {
    match std::env::var_os("SM83_TEST_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sm83-data/v1"),
    }
}

fn run_case(case: &Case) -> Result<(), String> {
    let mut memory = FlatMemory::new();
    for &(addr, value) in &case.initial.ram {
        memory.write(addr, value);
    }

    let mut cpu = Cpu::with_state(case.initial.cpu_state(), Box::new(memory));
    cpu.step().map_err(|e| format!("{}: {e}", case.name))?;

    let (expected, actual) = (case.end.cpu_state(), cpu.state());
    if actual != expected {
        return Err(format!(
            "{}: state mismatch\n     expected {expected:?}\n     actual   {actual:?}",
            case.name
        ));
    }

    for &(addr, value) in &case.end.ram {
        let got = cpu.read_memory(addr);
        if got != value {
            return Err(format!(
                "{}: ram[{addr:#06X}] expected {value:#04X}, got {got:#04X}",
                case.name
            ));
        }
    }

    Ok(())
}

#[test]
fn sm83_conformance() {
    let dir = test_dir();
    if !dir.is_dir() {
        eprintln!(
            "skipping sm83 conformance: no test data at {}\n\
             clone it there or set SM83_TEST_DIR (see the module docs)",
            dir.display()
        );
        return;
    }

    let mut opcodes = 0;
    let mut cases = 0;
    let mut failures = Vec::new();
    let mut failing_opcodes = Vec::new();

    for byte in 0u8..=0xFF {
        if Opcode::try_from(byte).is_err() {
            continue;
        }

        let path = dir.join(format!("{byte:02x}.json"));
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("opcode 0x{byte:02X} is implemented but {path:?}: {e}"));
        let opcode_cases: Vec<Case> =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("parsing {path:?}: {e}"));

        opcodes += 1;
        let mut opcode_failures = 0;
        for case in &opcode_cases {
            cases += 1;
            if let Err(message) = run_case(case) {
                if failures.len() < 20 {
                    failures.push(message);
                }
                opcode_failures += 1;
            }
        }
        if opcode_failures > 0 {
            failing_opcodes.push((byte, opcode_failures, opcode_cases.len()));
        }
    }

    println!("ran {cases} cases across {opcodes} implemented opcodes");

    if !failing_opcodes.is_empty() {
        let total: usize = failing_opcodes.iter().map(|&(_, f, _)| f).sum();
        for message in &failures {
            eprintln!("{message}");
        }
        eprintln!("\nfailing opcodes:");
        for (byte, failed, total) in &failing_opcodes {
            eprintln!("  0x{byte:02X}: {failed}/{total}");
        }
        panic!(
            "{total} of {cases} cases failed across {} opcodes",
            failing_opcodes.len()
        );
    }
}

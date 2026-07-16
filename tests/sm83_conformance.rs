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
use owo_colors::{OwoColorize, Stream};
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

fn diff(name: &str, want: String, got: String) -> String {
    format!(
        "{name} want {} got {}",
        want.if_supports_color(Stream::Stderr, |t| t.green()),
        got.if_supports_color(Stream::Stderr, |t| t.red())
    )
}

fn diff_states(want: &CpuState, got: &CpuState) -> Vec<String> {
    let mut diffs = Vec::new();
    for (name, want, got) in [("pc", want.pc, got.pc), ("sp", want.sp, got.sp)] {
        if want != got {
            diffs.push(diff(name, format!("{want:#06X}"), format!("{got:#06X}")));
        }
    }
    for (name, want, got) in [
        ("a", want.a, got.a),
        ("f", want.f, got.f),
        ("b", want.b, got.b),
        ("c", want.c, got.c),
        ("d", want.d, got.d),
        ("e", want.e, got.e),
        ("h", want.h, got.h),
        ("l", want.l, got.l),
    ] {
        if want != got {
            diffs.push(diff(name, format!("{want:#04X}"), format!("{got:#04X}")));
        }
    }
    if want.ime != got.ime {
        diffs.push(diff("ime", want.ime.to_string(), got.ime.to_string()));
    }
    diffs
}

fn run_case(case: &Case) -> Result<(), String> {
    let mut memory = FlatMemory::new();
    for &(addr, value) in &case.initial.ram {
        memory.write(addr, value);
    }

    let mut cpu = Cpu::with_state(case.initial.cpu_state(), Box::new(memory));
    cpu.step().map_err(|e| e.to_string())?;

    let mut diffs = diff_states(&case.end.cpu_state(), &cpu.state());
    for &(addr, value) in &case.end.ram {
        let got = cpu.read_memory(addr);
        if got != value {
            diffs.push(diff(
                &format!("ram[{addr:#06X}]"),
                format!("{value:#04X}"),
                format!("{got:#04X}"),
            ));
        }
    }

    if diffs.is_empty() {
        Ok(())
    } else {
        Err(diffs.join(" · "))
    }
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
    let mut failing = Vec::new();

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
        cases += opcode_cases.len();

        let mut failed = 0;
        let mut example = String::new();
        for case in &opcode_cases {
            if let Err(diff) = run_case(case) {
                if failed == 0 {
                    example = format!("e.g. {}: {diff}", case.name);
                }
                failed += 1;
            }
        }
        if failed > 0 {
            failing.push((byte, failed, opcode_cases.len(), example));
        }
    }

    if failing.is_empty() {
        let message = format!(
            "✅ sm83 conformance: all {cases} cases across {opcodes} implemented opcodes passed"
        );
        println!(
            "{}",
            message.if_supports_color(Stream::Stdout, |t| t.green())
        );
        return;
    }

    let failed_cases: usize = failing.iter().map(|&(_, failed, _, _)| failed).sum();
    let summary = format!(
        "sm83 conformance: {} of {opcodes} implemented opcodes failing ({failed_cases} of {cases} cases)",
        failing.len()
    );
    eprintln!();
    eprintln!(
        "{}",
        summary.if_supports_color(Stream::Stderr, |t| t.bold())
    );
    eprintln!();
    for (byte, failed, total, example) in &failing {
        let opcode = format!("0x{byte:02X}");
        eprintln!(
            "  ❌ {}  {failed:>4}/{total}  {example}",
            opcode.if_supports_color(Stream::Stderr, |t| t.bold())
        );
    }
    eprintln!();
    panic!(
        "{failed_cases} of {cases} cases failed across {} opcodes",
        failing.len()
    );
}

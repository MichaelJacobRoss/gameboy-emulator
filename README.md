# Game Boy Emulator

[![CI](https://github.com/MichaelJacobRoss/gameboy-emulator/actions/workflows/ci.yml/badge.svg)](https://github.com/MichaelJacobRoss/gameboy-emulator/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.95.0-orange.svg)](rust-toolchain.toml)

A pure-Rust emulator for the original Game Boy (DMG), built correctness-first: every implemented instruction is validated against third-party conformance suites in CI, not just spot-checked by hand.

> **Project status: early, actively developed.** The CPU core and memory bus are in place and roughly 20% of the instruction set is implemented and passing conformance tests. There is no graphics, sound, or cartridge support yet, so it does not run commercial games at this stage. The [roadmap](#roadmap) below tracks the path from here to a playable emulator. This README describes exactly what is and is not done today.

## Why this project

Most hobby emulators are validated by "it boots my favorite game." This one is built the other way around: the goal is to be *provably* correct against the same reference suites used to test production emulators, with that verification running automatically in CI. The interesting engineering here is the test infrastructure and the discipline it enforces, as much as the emulation itself.

Concretely, the design priorities are:

- **Verifiable correctness.** The [SingleStepTests/sm83](https://github.com/SingleStepTests/sm83) suite drives every implemented opcode against 1,000 reference cases each, asserting the full register and memory state afterwards.
- **A core that is a pure library.** The emulator is a dependency-light `lib` (only `thiserror` in the default build). Frontends, test harnesses, and future targets such as a browser build are all just consumers of it.
- **Reproducible builds.** The Rust toolchain is pinned (`rust-toolchain.toml`, edition 2024) with a Nix flake for a fully reproducible dev environment.

## Current status

| Subsystem | State | Notes |
|---|---|---|
| CPU register file and flags | Done | 8/16-bit views, `F` low-nibble masking enforced |
| Fetch / decode / execute pipeline | Done | Staged decode with exhaustiveness-checked dispatch |
| SM83 instruction set | In progress (~20%) | 103 of ~500 opcodes; see `cargo coverage` |
| Memory bus | Done | Full address map, `IE`/`IF` registers, serial stub |
| Boot-ROM handling | Done | Skipped via post-boot DMG register state |
| Interrupt dispatch | Planned | IME/IE/IF vectoring, HALT semantics |
| Cycle timing | Planned | Per-instruction M-cycle counting |
| Timer (DIV/TIMA/TMA/TAC) | Planned | |
| PPU (graphics) | Planned | Scanline renderer |
| Cartridge mappers (MBC1/3/5) | Planned | Needed to load most of the library |
| APU (sound) | Planned | |

Implemented instructions today: `NOP`, `JP a16`, the 8-bit load families (`LD r8,imm8`, `LD r8,r8`, the `(HL)` variants, `LD A,(a16)` / `LD (a16),A`), 16-bit immediate loads, `INC`/`DEC` for 8- and 16-bit registers, `DI`/`EI` (with the one-instruction enable delay), and `HALT`.

For a live, always-accurate breakdown of what is implemented, run the coverage checklist:

```sh
cargo coverage
```

It prints the full SM83 instruction set grouped in implementation order, marks each opcode done or pending by probing the decoder directly, and points at what to build next. The checklist can never drift from the code because it is derived from it.

## How correctness is proven

Three layers, from fastest to most exhaustive:

1. **Snapshot unit tests** (`cargo test`) cover each instruction family with hand-written cases, including flag edge cases. Fast, no external data.
2. **The coverage checklist** (`cargo coverage`) verifies which opcodes decode and keeps the roadmap honest.
3. **The SingleStepTests conformance harness** (`cargo conformance`) is the real proof. It runs 1,000 reference cases per opcode against the CPU and a flat memory, comparing final register state (including `IME`) and RAM byte-for-byte. Adding an opcode automatically pulls its cases into the run, so coverage grows for free.

CI runs formatting (`rustfmt`), lints (`clippy`, all targets and features), a warnings-as-errors build, the unit tests, and the full conformance suite on every push. The conformance job downloads and caches the ~500 MB reference corpus.

## Architecture

The core is split so that decoding, operand fetch, and execution are separate, type-checked stages:

```
byte --> Opcode --> Instruction --> execute
         (decode    (operands       (mutate CPU
          first      fetched)         + memory)
          byte)
```

- `Opcode` is the pure decode of the first byte, using bitmask patterns that mirror the hardware's `0b_ooo_sss` operand encoding rather than a hand-written 256-entry table. This stage needs no CPU state, which is what lets the coverage and conformance harnesses cheaply ask "is this byte implemented?"
- `Instruction` carries any fetched immediates.
- `execute` applies the effect. Flag changes go through a single `FlagAdjustment` helper so "unaffected" is encoded in the type rather than forgotten.

Every stage is a `match` the compiler checks for exhaustiveness, so an unhandled opcode is a build error, not a silent bug.

Module layout:

```
src/
  cpu.rs            fetch/decode/execute loop, CPU state
  cpu/registers.rs  register file, 8/16-bit access, reg! macro
  cpu/flags.rs      Z/N/H/C flag access
  instruction/      opcode decode, operand decode, execution, per-family tests
  memory/           memory bus trait, full address map, flat test memory
tests/
  sm83_conformance.rs   SingleStepTests harness
```

## Building and testing

Requires the pinned Rust toolchain (installed automatically by `rustup` from `rust-toolchain.toml`), or `nix develop` for a reproducible shell.

```sh
# Build
cargo build

# Unit tests
cargo test

# Instruction-set coverage checklist
cargo coverage

# Full conformance suite (downloads the reference corpus on first run)
git clone --depth 1 https://github.com/SingleStepTests/sm83 sm83-data
cargo conformance
```

## Roadmap

The project follows a dependency-ordered plan (full detail in [`ROADMAP.md`](ROADMAP.md)). Each milestone is a demonstrable step:

| Milestone | Goal |
|---|---|
| M1 | Blargg `cpu_instrs` sub-tests print "Passed" (full instruction set) |
| M2 | Combined `cpu_instrs` + `instr_timing` pass (cycle timing, interrupts, timer) |
| M3 | Tetris title screen renders (PPU background + window) |
| M4 | Tetris fully playable (sprites, OAM DMA, joypad, frame pacing) |
| M5 | dmg-acid2 pixel-perfect (framebuffer-hash regression test) |
| M6 | Pokémon Red playable with persistent saves (cartridge mappers) |
| M7 | Audio on, Tetris theme correct (APU) |
| M8 | CI scorecard, gameplay GIF, browser (wasm) demo |

Currently working toward **M1**.

## References

Built against the community's reference documentation:

- [Pan Docs](https://gbdev.io/pandocs/): the definitive DMG hardware reference
- [gbops opcode table](https://izik1.github.io/gbops/): instruction encodings, sizes, and cycle counts
- [RGBDS gbz80(7)](https://rgbds.gbdev.io/docs/gbz80.7): per-instruction semantics
- [SingleStepTests/sm83](https://github.com/SingleStepTests/sm83): the conformance corpus

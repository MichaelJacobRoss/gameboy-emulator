#![cfg(feature = "opcode-coverage")]

//! A living checklist of the full SM83 instruction set.
//!
//! Run it with:
//!
//! ```text
//! cargo coverage
//! ```
//!
//! (an alias in `.cargo/config.toml` for
//! `cargo test --features opcode-coverage opcode_coverage -- --nocapture`)
//!
//! Sections follow a sensible implementation order for a fresh CPU core;
//! implementation status is probed from the decoder, so the catalog itself
//! never needs updating. The test fails while opcodes are missing (the
//! failure output includes the checklist) and passes once the whole set
//! decodes. The catalog is verified for completeness, so every valid opcode
//! appears exactly once.
//!
//! Unfinished groups link to the matching section of the RGBDS gbz80(7)
//! instruction reference (<https://rgbds.gbdev.io/docs/gbz80.7>).

use std::collections::HashSet;
use std::iter::once;

use comfy_table::{Cell, CellAlignment, Table, presets::NOTHING};

use crate::instruction::Opcode;

const R8: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
const R16: [&str; 4] = ["BC", "DE", "HL", "SP"];
const R16_STK: [&str; 4] = ["BC", "DE", "HL", "AF"];
const CC: [&str; 4] = ["NZ", "Z", "NC", "C"];
const ALU: [&str; 8] = ["ADD", "ADC", "SUB", "SBC", "AND", "XOR", "OR", "CP"];
const CB_SHIFT: [&str; 8] = ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SWAP", "SRL"];

/// Per-instruction reference; a group's doc links are `DOCS_URL#anchor`.
const DOCS_URL: &str = "https://rgbds.gbdev.io/docs/gbz80.7";

/// Opcode-byte grid (encodings, sizes, cycle counts) — one page covers all.
const OPCODE_TABLE_URL: &str = "https://izik1.github.io/gbops/";

/// Bytes with no instruction behind them: 0xCB is the prefix byte, and the
/// other eleven hard-lock real hardware, so they must keep decoding as errors.
const NON_INSTRUCTIONS: [u8; 12] = [
    0xCB, 0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
];

struct Entry {
    cb: bool,
    byte: u8,
    mnemonic: String,
}

struct Group {
    name: String,
    docs: Vec<String>,
    entries: Vec<Entry>,
}

struct Section {
    title: &'static str,
    groups: Vec<Group>,
}

fn base(byte: u8, mnemonic: impl Into<String>) -> Entry {
    Entry {
        cb: false,
        byte,
        mnemonic: mnemonic.into(),
    }
}

fn cb(byte: u8, mnemonic: impl Into<String>) -> Entry {
    Entry {
        cb: true,
        byte,
        mnemonic: mnemonic.into(),
    }
}

fn group(name: &str, docs: &[&str], entries: Vec<Entry>) -> Group {
    Group {
        name: name.to_string(),
        docs: docs.iter().map(|anchor| anchor.to_string()).collect(),
        entries,
    }
}

fn is_implemented(entry: &Entry) -> bool {
    if entry.cb {
        // TODO: probe the CB decode path here once it exists,
        // e.g. `CbOpcode::try_from(entry.byte).is_ok()`.
        false
    } else {
        Opcode::try_from(entry.byte).is_ok()
    }
}

/// The eight `r8` operand indexes minus 6, which encodes `(HL)`.
fn r8_regs() -> impl Iterator<Item = u8> {
    (0..8).filter(|&i| i != 6)
}

fn catalog() -> Vec<Section> {
    vec![
        Section {
            title: "§1 First steps",
            groups: vec![
                group("NOP", &["NOP"], vec![base(0x00, "NOP")]),
                group("JP a16", &["JP_n16"], vec![base(0xC3, "JP a16")]),
            ],
        },
        Section {
            title: "§2 8-bit loads",
            groups: vec![
                group(
                    "LD r8, imm8",
                    &["LD_r8,n8", "LD__HL_,n8"],
                    (0..8)
                        .map(|d| base(d << 3 | 0x06, format!("LD {},imm8", R8[d as usize])))
                        .collect(),
                ),
                group(
                    "LD r8, r8",
                    &["LD_r8,r8"],
                    r8_regs()
                        .flat_map(|d| {
                            r8_regs().map(move |s| {
                                base(
                                    0x40 | d << 3 | s,
                                    format!("LD {},{}", R8[d as usize], R8[s as usize]),
                                )
                            })
                        })
                        .collect(),
                ),
                group(
                    "LD (HL), r8",
                    &["LD__HL_,r8"],
                    r8_regs()
                        .map(|s| base(0x70 | s, format!("LD (HL),{}", R8[s as usize])))
                        .collect(),
                ),
                group(
                    "LD r8, (HL)",
                    &["LD_r8,_HL_"],
                    r8_regs()
                        .map(|d| base(0x46 | d << 3, format!("LD {},(HL)", R8[d as usize])))
                        .collect(),
                ),
                group(
                    "LD A <-> (BC)/(DE)",
                    &["LD__r16_,A", "LD_A,_r16_"],
                    vec![
                        base(0x02, "LD (BC),A"),
                        base(0x0A, "LD A,(BC)"),
                        base(0x12, "LD (DE),A"),
                        base(0x1A, "LD A,(DE)"),
                    ],
                ),
                group(
                    "LD A <-> (HL+)/(HL-)",
                    &["LD__HLI_,A", "LD_A,_HLI_", "LD__HLD_,A", "LD_A,_HLD_"],
                    vec![
                        base(0x22, "LD (HL+),A"),
                        base(0x2A, "LD A,(HL+)"),
                        base(0x32, "LD (HL-),A"),
                        base(0x3A, "LD A,(HL-)"),
                    ],
                ),
                group(
                    "LD A <-> (a16)",
                    &["LD_A,_n16_", "LD__n16_,A"],
                    vec![base(0xFA, "LD A,(a16)"), base(0xEA, "LD (a16),A")],
                ),
                group(
                    "LDH (0xFF00 page)",
                    &["LDH__n16_,A", "LDH_A,_n16_", "LDH__C_,A", "LDH_A,_C_"],
                    vec![
                        base(0xE0, "LDH (a8),A"),
                        base(0xF0, "LDH A,(a8)"),
                        base(0xE2, "LDH (C),A"),
                        base(0xF2, "LDH A,(C)"),
                    ],
                ),
            ],
        },
        Section {
            title: "§3 16-bit loads & stack",
            groups: vec![
                group(
                    "LD r16, imm16",
                    &["LD_r16,n16"],
                    (0..4)
                        .map(|i| base(i << 4 | 0x01, format!("LD {},imm16", R16[i as usize])))
                        .collect(),
                ),
                group(
                    "Stack-pointer loads",
                    &["LD__n16_,SP", "LD_SP,HL"],
                    vec![base(0x08, "LD (a16),SP"), base(0xF9, "LD SP,HL")],
                ),
                group(
                    "PUSH/POP r16",
                    &["PUSH_r16", "POP_r16"],
                    (0..4)
                        .map(|i| base(0xC5 | i << 4, format!("PUSH {}", R16_STK[i as usize])))
                        .chain(
                            (0..4).map(|i| {
                                base(0xC1 | i << 4, format!("POP {}", R16_STK[i as usize]))
                            }),
                        )
                        .collect(),
                ),
            ],
        },
        Section {
            title: "§4 8-bit arithmetic & logic",
            groups: once(group(
                "INC/DEC r8",
                &["INC_r8", "DEC_r8", "INC__HL_", "DEC__HL_"],
                (0..8)
                    .map(|d| base(d << 3 | 0x04, format!("INC {}", R8[d as usize])))
                    .chain((0..8).map(|d| base(d << 3 | 0x05, format!("DEC {}", R8[d as usize]))))
                    .collect(),
            ))
            .chain(ALU.iter().enumerate().map(|(op, alu)| {
                Group {
                    name: format!("{alu} A, r8/(HL)"),
                    docs: vec![format!("{alu}_A,r8"), format!("{alu}_A,_HL_")],
                    entries: (0..8)
                        .map(|s| {
                            base(
                                0x80 | (op as u8) << 3 | s,
                                format!("{alu} A,{}", R8[s as usize]),
                            )
                        })
                        .collect(),
                }
            }))
            .chain(once(Group {
                name: "ALU A, imm8".to_string(),
                docs: ALU.iter().map(|alu| format!("{alu}_A,n8")).collect(),
                entries: ALU
                    .iter()
                    .enumerate()
                    .map(|(op, alu)| base(0xC6 | (op as u8) << 3, format!("{alu} A,imm8")))
                    .collect(),
            }))
            .collect(),
        },
        Section {
            title: "§5 16-bit arithmetic",
            groups: vec![
                group(
                    "INC/DEC r16",
                    &["INC_r16", "DEC_r16"],
                    (0..4)
                        .map(|i| base(i << 4 | 0x03, format!("INC {}", R16[i as usize])))
                        .chain(
                            (0..4).map(|i| base(i << 4 | 0x0B, format!("DEC {}", R16[i as usize]))),
                        )
                        .collect(),
                ),
                group(
                    "ADD HL, r16",
                    &["ADD_HL,r16"],
                    (0..4)
                        .map(|i| base(i << 4 | 0x09, format!("ADD HL,{}", R16[i as usize])))
                        .collect(),
                ),
                group(
                    "SP + signed offset",
                    &["ADD_SP,e8", "LD_HL,SP+e8"],
                    vec![base(0xE8, "ADD SP,e8"), base(0xF8, "LD HL,SP+e8")],
                ),
            ],
        },
        Section {
            title: "§6 Jumps, calls & returns",
            groups: vec![
                group(
                    "JR (relative jumps)",
                    &["JR_n16", "JR_cc,n16"],
                    once(base(0x18, "JR e8"))
                        .chain(
                            (0..4)
                                .map(|c| base(0x20 | c << 3, format!("JR {},e8", CC[c as usize]))),
                        )
                        .collect(),
                ),
                group(
                    "JP cc / JP HL",
                    &["JP_cc,n16", "JP_HL"],
                    (0..4)
                        .map(|c| base(0xC2 | c << 3, format!("JP {},a16", CC[c as usize])))
                        .chain(once(base(0xE9, "JP HL")))
                        .collect(),
                ),
                group(
                    "CALL",
                    &["CALL_n16", "CALL_cc,n16"],
                    once(base(0xCD, "CALL a16"))
                        .chain(
                            (0..4).map(|c| {
                                base(0xC4 | c << 3, format!("CALL {},a16", CC[c as usize]))
                            }),
                        )
                        .collect(),
                ),
                group(
                    "RET / RETI",
                    &["RET", "RET_cc", "RETI"],
                    once(base(0xC9, "RET"))
                        .chain(
                            (0..4).map(|c| base(0xC0 | c << 3, format!("RET {}", CC[c as usize]))),
                        )
                        .chain(once(base(0xD9, "RETI")))
                        .collect(),
                ),
                group(
                    "RST",
                    &["RST_vec"],
                    (0..8)
                        .map(|n| base(0xC7 | n << 3, format!("RST 0x{:02X}", n * 8)))
                        .collect(),
                ),
            ],
        },
        Section {
            title: "§7 Rotates, DAA & CPU control",
            groups: vec![
                group(
                    "Accumulator rotates",
                    &["RLCA", "RRCA", "RLA", "RRA"],
                    vec![
                        base(0x07, "RLCA"),
                        base(0x0F, "RRCA"),
                        base(0x17, "RLA"),
                        base(0x1F, "RRA"),
                    ],
                ),
                group(
                    "DAA / CPL / SCF / CCF",
                    &["DAA", "CPL", "SCF", "CCF"],
                    vec![
                        base(0x27, "DAA"),
                        base(0x2F, "CPL"),
                        base(0x37, "SCF"),
                        base(0x3F, "CCF"),
                    ],
                ),
                group(
                    "Interrupt & power control",
                    &["DI", "EI", "HALT", "STOP"],
                    vec![
                        base(0xF3, "DI"),
                        base(0xFB, "EI"),
                        base(0x76, "HALT"),
                        base(0x10, "STOP"),
                    ],
                ),
            ],
        },
        Section {
            title: "§8 CB-prefixed page",
            groups: CB_SHIFT
                .iter()
                .enumerate()
                .map(|(op, name)| Group {
                    name: format!("{name} r8/(HL)"),
                    docs: vec![format!("{name}_r8"), format!("{name}__HL_")],
                    entries: (0..8)
                        .map(|s| cb((op as u8) << 3 | s, format!("{name} {}", R8[s as usize])))
                        .collect(),
                })
                .chain(
                    [(0x40u8, "BIT"), (0x80, "RES"), (0xC0, "SET")].map(|(block, name)| Group {
                        name: format!("{name} b, r8/(HL)"),
                        docs: vec![format!("{name}_u3,r8"), format!("{name}_u3,_HL_")],
                        entries: (0..8)
                            .flat_map(|b| {
                                (0..8).map(move |s| {
                                    cb(block | b << 3 | s, format!("{name} {b},{}", R8[s as usize]))
                                })
                            })
                            .collect(),
                    }),
                )
                .collect(),
        },
    ]
}

/// Every valid opcode must appear in the catalog exactly once, and no
/// non-instruction byte may sneak in — otherwise the checklist lies.
fn assert_catalog_is_complete(sections: &[Section]) {
    let mut base_seen = HashSet::new();
    let mut cb_seen = HashSet::new();

    for entry in sections
        .iter()
        .flat_map(|s| &s.groups)
        .flat_map(|g| &g.entries)
    {
        let seen = if entry.cb {
            &mut cb_seen
        } else {
            &mut base_seen
        };
        assert!(
            seen.insert(entry.byte),
            "catalog lists {}0x{:02X} twice",
            if entry.cb { "CB " } else { "" },
            entry.byte
        );
    }

    for byte in 0..=0xFFu8 {
        if NON_INSTRUCTIONS.contains(&byte) {
            assert!(
                !base_seen.contains(&byte),
                "0x{byte:02X} is not an instruction but the catalog lists it"
            );
        } else {
            assert!(
                base_seen.contains(&byte),
                "catalog is missing base opcode 0x{byte:02X}"
            );
        }
        assert!(
            cb_seen.contains(&byte),
            "catalog is missing CB opcode CB 0x{byte:02X}"
        );
    }
}

fn tally(entries: &[Entry]) -> (usize, usize) {
    let done = entries.iter().filter(|entry| is_implemented(entry)).count();
    (done, entries.len())
}

fn progress_bar(done: usize, total: usize, width: usize) -> String {
    let filled = (done * width).checked_div(total).unwrap_or(width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

fn group_mark(done: usize, total: usize) -> &'static str {
    if done == total {
        "✅"
    } else if done == 0 {
        "⬜"
    } else {
        "🚧"
    }
}

/// Emoji are double-width in the terminal but single chars to `format!`,
/// so all alignment goes through comfy-table, which measures display width.
fn table_lines(table: &Table) -> Vec<String> {
    table.to_string().lines().map(str::to_string).collect()
}

fn group_summary_lines(groups: &[Group]) -> Vec<String> {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    for grp in groups {
        let (done, total) = tally(&grp.entries);
        table.add_row(vec![
            Cell::new(group_mark(done, total)),
            Cell::new(&grp.name),
            Cell::new(format!("{done}/{total}")).set_alignment(CellAlignment::Right),
            Cell::new(progress_bar(done, total, 16)),
        ]);
    }
    table_lines(&table)
}

/// One line per group: the first doc anchor as a clickable URL, further
/// instruction variants as bare `#fragment`s onto the same page.
fn docs_line(docs: &[String]) -> Option<String> {
    let (first, rest) = docs.split_first()?;
    let mut line = format!("↪ {DOCS_URL}#{first}");
    if !rest.is_empty() {
        let fragments: Vec<String> = rest.iter().map(|anchor| format!("#{anchor}")).collect();
        line.push_str(&format!("  (also {})", fragments.join(" ")));
    }
    Some(line)
}

fn entry_grid_lines(entries: &[Entry]) -> Vec<String> {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    for row in entries.chunks(4) {
        table.add_row(row.iter().map(|e| {
            let mark = if is_implemented(e) { "✅" } else { "⬜" };
            let prefix = if e.cb { "CB" } else { "0x" };
            format!("{mark} {prefix}{:02X} {}", e.byte, e.mnemonic)
        }));
    }
    table_lines(&table)
}

#[test]
fn opcode_coverage() {
    let sections = catalog();
    assert_catalog_is_complete(&sections);

    let mut done_overall = 0;
    let mut total_overall = 0;
    let mut next_up: Option<String> = None;

    println!();
    println!("SM83 instruction-set checklist — in suggested implementation order");
    println!("Opcode table: {OPCODE_TABLE_URL}");

    for section in &sections {
        let (sec_done, sec_total) = section
            .groups
            .iter()
            .map(|g| tally(&g.entries))
            .fold((0, 0), |(d, t), (gd, gt)| (d + gd, t + gt));
        done_overall += sec_done;
        total_overall += sec_total;

        println!();
        println!(
            "{:<44} {:>3}/{:<3} {}",
            section.title,
            sec_done,
            sec_total,
            progress_bar(sec_done, sec_total, 20)
        );
        println!("{}", "─".repeat(74));

        let summary = group_summary_lines(&section.groups);
        for (grp, line) in section.groups.iter().zip(summary) {
            println!("  {line}");
            let (done, total) = tally(&grp.entries);
            if done < total {
                if let Some(docs) = docs_line(&grp.docs) {
                    println!("         {docs}");
                }
                for grid_line in entry_grid_lines(&grp.entries) {
                    println!("        {grid_line}");
                }
                if next_up.is_none() {
                    next_up = Some(format!("{} — {}", section.title, grp.name));
                }
            }
        }
    }

    println!();
    println!("{}", "═".repeat(74));
    let percent = done_overall as f64 * 100.0 / total_overall as f64;
    println!(
        "Overall: {done_overall}/{total_overall} ({percent:.1}%)  {}",
        progress_bar(done_overall, total_overall, 40)
    );
    println!("Not counted: the 0xCB prefix byte and 11 illegal opcodes (must stay errors).");
    match &next_up {
        Some(next) => println!("Next up ➜ {next}"),
        None => println!("Instruction set complete — go run blargg's cpu_instrs 🎉"),
    }
    println!();

    assert_eq!(
        done_overall, total_overall,
        "instruction set incomplete ({done_overall}/{total_overall}) — the checklist above shows what's left"
    );
}

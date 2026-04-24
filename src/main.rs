// Temporary for early development
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unreachable_patterns)]

mod cpu;
mod instruction;
mod memory;
mod util;

use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom.gb>", args[0]);
        process::exit(1);
    }

    let rom = fs::read(&args[1]).unwrap_or_else(|e| {
        eprintln!("Failed to read '{}': {}", args[1], e);
        process::exit(1);
    });

    let bus = memory::Bus::new(rom);
    let mut cpu = cpu::Cpu::new_dmg(Box::new(bus));

    loop {
        if let Err(e) = cpu.step() {
            eprintln!("\nCPU error: {}", e);
            process::exit(1);
        }
    }
}

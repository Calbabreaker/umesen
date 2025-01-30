use std::fs::File;

use umesen_core::{Cartridge, Emulator};

// Test rom obtained from https://github.com/christopherpow/nes-test-roms
#[test]
fn scanline() {
    let rom_file = File::open("tests/scanline.nes").unwrap();
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus
        .attach_catridge(umesen_core::Cartridge::from_nes(rom_file).unwrap());
    cpu.reset();
    for i in 0..1000 {
        cpu.execute_next().unwrap();
    }
}

// Test rom by kevtris https://www.qmtpro.com/~nes/misc/nestest.txt
#[test]
fn nestest() {
    let correct_logs = include_str!("nestest.log");

    let mut emu = Emulator::default();
    let cartridge = Cartridge::from_nes(&include_bytes!("nestest.nes")[..]).unwrap();
    emu.cpu.bus.attach_catridge(cartridge);
    emu.cpu.reset();
    emu.cpu.pc = 0xc000;

    let mut prev_disassem = "CPU RESET";
    for (i, correct_line) in correct_logs.lines().enumerate() {
        let mut line_split = correct_line.split("//");
        let correct_output = line_split.next().unwrap().trim();
        assert_eq!(
            emu.get_debug_log(),
            correct_output,
            "Incorrect output on line {i} executing: {prev_disassem}"
        );

        // Current line actually contains the state of the instruction executed last line
        prev_disassem = line_split.next().unwrap().trim();

        if let Err(err) = emu.cpu.execute_next() {
            panic!("Failed to execute at line {i}: {err} ({prev_disassem})");
        }
    }
}

// https://github.com/christopherpow/nes-test-roms/blob/master/instr_test-v5/readme.txt
#[test]
#[ignore = "Need mapper 001"]
fn instr_test() {
    let mut emu = Emulator::default();
    emu.load_nes_rom("tests/instr_test_official.nes").unwrap();
}

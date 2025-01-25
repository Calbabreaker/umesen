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

    testing_logger::setup();
    for _ in 0..correct_logs.lines().count() {
        emu.cpu.execute_next().unwrap();
    }

    testing_logger::validate(|captured_logs| {
        let log_lines = captured_logs
            .iter()
            .filter(|log| log.level == log::Level::Trace && log.target == "cpu-debug")
            .map(|log| log.body.clone())
            .collect::<Vec<_>>();

        for (i, correct_line) in correct_logs.lines().enumerate() {
            let mut line_split = correct_line.split("//");
            let correct_line = line_split.next().unwrap().trim();
            let disassem = line_split.next().unwrap();
            assert_eq!(
                correct_line, log_lines[i],
                "Didn't match on line {i}: {disassem}"
            );
        }
    });
}

// https://github.com/christopherpow/nes-test-roms/blob/master/instr_test-v5/readme.txt
#[test]
#[ignore]
fn instr_test() {
    let mut emu = Emulator::default();
    emu.load_nes_rom("tests/instr_test_official.nes").unwrap();
}

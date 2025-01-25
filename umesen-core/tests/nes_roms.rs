use std::fs::File;

use umesen_core::Emulator;

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
    let mut emu = Emulator::default();
    emu.load_nes_rom("tests/nestest.nes").unwrap();
    emu.cpu.pc = 0xc000;
}

// https://github.com/christopherpow/nes-test-roms/blob/master/instr_test-v5/readme.txt
#[test]
#[ignore]
fn instr_test() {
    let mut emu = Emulator::default();
    emu.load_nes_rom("tests/instr_test_official.nes").unwrap();
}

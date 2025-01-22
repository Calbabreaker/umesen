use std::fs::File;

// Test rom obtained from https://github.com/christopherpow/nes-test-roms
#[test]
fn scanline() {
    let rom_file = File::open("tests/scanline.nes").unwrap();
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus.cartridge = Some(umesen_core::Cartridge::from_nes(rom_file).unwrap());
    cpu.reset();
    for i in 0..1000 {
        cpu.execute_next().unwrap();
    }
}

// Test rom by kevtris https://www.qmtpro.com/~nes/misc/nestest.txt
#[test]
fn nestest() {}

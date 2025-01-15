use std::fs::File;

// Test roms obtained from https://github.com/christopherpow/nes-test-roms

#[test]
#[ignore]
fn cpu_dummy_reads() {
    let rom_file = File::open("tests/cpu_dummy_reads.nes").unwrap();
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus.cartridge = Some(umesen_core::Catridge::from_nes(rom_file).unwrap());
    cpu.execute_next().unwrap();
}

#[test]
fn scanline() {
    let rom_file = File::open("tests/scanline.nes").unwrap();
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus.cartridge = Some(umesen_core::Catridge::from_nes(rom_file).unwrap());
    cpu.reset();
    for i in 0..1000 {
        cpu.execute_next().unwrap();
    }

    panic!();
}

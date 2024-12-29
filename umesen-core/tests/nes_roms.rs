use std::fs::File;

#[test]
fn cpu_dummy_reads() {
    let rom_file = File::open("tests/cpu_dummy_reads.nes").unwrap();
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus.cartridge = Some(umesen_core::Cartridge::from_nes(rom_file).unwrap());
    cpu.execute_next().unwrap();
}

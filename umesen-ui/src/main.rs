fn main() {
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus.cartridge = Some(umesen_core::cartridge::new_only_ram(1024));
    cpu.bus.write_byte(0xffff, 0x6);
    cpu.execute_next().unwrap();
}

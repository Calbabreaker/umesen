fn main() {
    let mut cpu = umesen_core::Cpu::default();
    cpu.bus.write_byte(0, 0x6);
    cpu.execute_next().unwrap();
}

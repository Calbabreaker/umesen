#[derive(thiserror::Error, Debug)]
pub enum CpuError {
    #[error("unknown instruction with opcode: 0x{0:02x}")]
    UnknownOpcode(u8),
}

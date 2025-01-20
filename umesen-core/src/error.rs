#[derive(thiserror::Error, Debug)]
pub enum CpuError {
    #[error("unknown instruction with opcode: 0x{0:02x}")]
    UnknownOpcode(u8),
}

#[derive(thiserror::Error, Debug)]
pub enum NesParseError {
    #[error("magic number '{0}' in header is not a valid NES header")]
    InvalidMagicNumber(String),
    #[error("mapper id '{0}' is not supported")]
    UnsupportedMapper(u8),
    #[error("expected at least {0} bytes of data")]
    NotEnough(usize),
}

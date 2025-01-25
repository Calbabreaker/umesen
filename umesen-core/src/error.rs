#[derive(thiserror::Error, Debug)]
pub enum CpuError {
    #[error("Unknown instruction with opcode: 0x{0:02x}")]
    UnknownOpcode(u8),
    #[error("Debug trap was tripped")]
    DebugTrap,
}

#[derive(thiserror::Error, Debug)]
pub enum NesParseError {
    #[error("Magic number '{0}' in header is not a valid NES header")]
    InvalidMagicNumber(String),
    #[error("Mapper id '{0}' is not supported")]
    UnsupportedMapper(u8),
    #[error("Expected at least {0} bytes of data")]
    NotEnough(usize),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

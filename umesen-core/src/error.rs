#[derive(thiserror::Error, Debug)]
pub enum NesParseError {
    #[error("Magic number '{0}' in header is not a valid NES header")]
    InvalidMagicNumber(String),
    #[error("Mapper id '{0}' is not supported")]
    UnsupportedMapper(u16),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

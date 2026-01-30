use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected byte {0}")]
    UnexpectedByte(u8),

    #[error("Expected byte {expected}, got {actual}")]
    ExpectedByte { actual: u8, expected: u8 },

    #[error("Unexpected EOF")]
    UnexpectedEOF,

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("Error parsing integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

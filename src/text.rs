//! Shared bounds and failure classification for printable diagnostic metadata.

use crate::constants::DIAGNOSTIC_STRING_MAX_BYTES;

///
/// DiagnosticTextError
///
/// Field-independent failure mapped to each boundary's existing public error.
///

#[derive(Clone, Copy)]
pub enum DiagnosticTextError {
    Empty,
    TooLong,
    NonAscii,
    ControlCharacter,
}

impl DiagnosticTextError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Empty => "must not be empty",
            Self::TooLong => "must be at most 256 bytes",
            Self::NonAscii => "must be ASCII",
            Self::ControlCharacter => "must not contain ASCII control characters",
        }
    }
}

pub fn validate_diagnostic_text(value: &str) -> Result<(), DiagnosticTextError> {
    if value.is_empty() {
        return Err(DiagnosticTextError::Empty);
    }
    if value.len() > DIAGNOSTIC_STRING_MAX_BYTES {
        return Err(DiagnosticTextError::TooLong);
    }
    if !value.is_ascii() {
        return Err(DiagnosticTextError::NonAscii);
    }
    if value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DiagnosticTextError::ControlCharacter);
    }
    Ok(())
}

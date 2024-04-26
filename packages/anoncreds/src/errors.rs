use thiserror::Error;
use ursa::errors::UrsaCryptoError;

#[derive(Error, Debug, PartialEq)]
pub enum TypeConversionError {
    #[error("UrsaCryptoError: {0}")]
    UrsaCryptoError(String),

    #[error("Conversion: {0}")]
    Conversion(String),
}

impl From<UrsaCryptoError> for TypeConversionError {
    fn from(source: UrsaCryptoError) -> Self {
        Self::UrsaCryptoError(source.to_string())
    }
}

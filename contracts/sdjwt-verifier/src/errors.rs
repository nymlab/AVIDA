use avida_cheqd::ibc::ChannelError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use serde_json::error::Error as SerdeJsonError;
use thiserror::Error;

#[cw_serde]
pub enum SdjwtVerifierResultError {
    PresentationTooLarge,
    VerifiedClaimsTypeUnexpected,
    CriterionValueTypeUnexpected,
    CriterionValueNumberInvalid,
    CriterionValueFailed(String),
    DisclosedClaimNotFound(String),
    RequiredClaimsNotSatisfied,
    PubKeyNotFound,
    JwtError(String),
    StringConversion(String),
    SdJwt(String),
    ExpirationStringInvalid(String),
    ExpirationKeyOrValueInvalid(String, String),
    PresentationExpired(cw_utils::Expiration),
    IdxRevoked(u64),
}

impl std::fmt::Display for SdjwtVerifierResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdjwtVerifierResultError::PresentationTooLarge => write!(f, "Presentation too large"),
            SdjwtVerifierResultError::VerifiedClaimsTypeUnexpected => {
                write!(f, "Verified claims type unexpected")
            }
            SdjwtVerifierResultError::CriterionValueTypeUnexpected => {
                write!(f, "Criterion value type unexpected")
            }
            SdjwtVerifierResultError::CriterionValueNumberInvalid => {
                write!(f, "Criterion value number invalid")
            }
            SdjwtVerifierResultError::CriterionValueFailed(msg) => {
                write!(f, "Criterion value failed: {}", msg)
            }
            SdjwtVerifierResultError::DisclosedClaimNotFound(msg) => {
                write!(f, "Disclosed claim not found: {}", msg)
            }
            SdjwtVerifierResultError::RequiredClaimsNotSatisfied => {
                write!(f, "Required claims not satisfied")
            }
            SdjwtVerifierResultError::PubKeyNotFound => write!(f, "Public key not found"),
            SdjwtVerifierResultError::JwtError(msg) => write!(f, "JWT error: {}", msg),
            SdjwtVerifierResultError::StringConversion(msg) => {
                write!(f, "String conversion error: {}", msg)
            }
            SdjwtVerifierResultError::SdJwt(msg) => write!(f, "SD-JWT error: {}", msg),
            SdjwtVerifierResultError::ExpirationStringInvalid(msg) => {
                write!(f, "Expiration string invalid: {}", msg)
            }
            SdjwtVerifierResultError::ExpirationKeyOrValueInvalid(key, value) => {
                write!(f, "Expiration key or value invalid: {} - {}", key, value)
            }
            SdjwtVerifierResultError::PresentationExpired(exp) => {
                write!(f, "Presentation expired: {:?}", exp)
            }
            SdjwtVerifierResultError::IdxRevoked(idx) => write!(f, "IDX revoked: {}", idx),
        }
    }
}

#[derive(Error, Debug)]
pub enum SdjwtVerifierError {
    #[error("Verifier Result Error {0}")]
    SdjwtVerifierResultError(SdjwtVerifierResultError),
    #[error("SudoValidationFailed")]
    SudoValidationFailed,
    #[error("IBC returned resource format unexpected {0}")]
    ReturnedResourceFormat(String),
    #[error("IBC channel already exists")]
    ChannelAlreadyExists,
    #[error("Serde JSON Error {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("channel error")]
    ChannelError(#[from] ChannelError),
    #[error("data deserialization error")]
    DataDeserialization,
    #[error("App Already Registered")]
    AppAlreadyRegistered,
    #[error("App Is Not Registered")]
    AppIsNotRegistered,
    #[error("Unauthorised")]
    Unauthorised,
    #[error("Unsupported Key Type")]
    UnsupportedKeyType,
    #[error("Route Not Registered")]
    RouteNotRegistered,
    #[error("No Requirements For Route")]
    NoRequirementsForRoute,
    #[error("IDX Not In Requirement")]
    IDXNotInRequirement,
    #[error("Revocation List type")]
    RevocationListType,
}

impl From<SdjwtVerifierError> for StdError {
    fn from(err: SdjwtVerifierError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}

use avida_cheqd::ibc::ChannelError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use sd_jwt_rs::error::Error as SdJwtRsError;
use serde_json::error::Error as SerdeJsonError;
use thiserror::Error;

#[cw_serde]
#[derive(Error)]
pub enum SdjwtVerifierResultError {
    #[error("Presentation too large")]
    PresentationTooLarge,
    #[error("Verified claims type unexpected")]
    VerifiedClaimsTypeUnexpected,
    #[error("Criterion value type unexpected")]
    CriterionValueTypeUnexpected,
    #[error("Criterion value number invalid")]
    CriterionValueNumberInvalid,
    #[error("Criterion value failed: {0}")]
    CriterionValueFailed(String),
    #[error("Disclosed claim not found: {0}")]
    DisclosedClaimNotFound(String),
    #[error("Required claims not satisfied")]
    RequiredClaimsNotSatisfied,
    #[error("Public key not found")]
    PubKeyNotFound,
    #[error("JWT error: {0}")]
    JwtError(String),
    #[error("String conversion error: {0}")]
    StringConversion(String),
    #[error("SD-JWT error: {0}")]
    SdJwt(String),
    #[error("Expiration string invalid: {0}")]
    ExpirationStringInvalid(String),
    #[error("Expiration key or value invalid: {0} - {1}")]
    ExpirationKeyOrValueInvalid(String, String),
    #[error("Presentation expired: {0:?}")]
    PresentationExpired(cw_utils::Expiration),
    #[error("IDX revoked: {0}")]
    IdxRevoked(u64),
    #[error("Issuer not found in Payload")]
    IssuerNotFound,
    #[error("SdJwtRsError: {0}")]
    SdJwtRsError(String),
}

impl From<SdJwtRsError> for SdjwtVerifierResultError {
    fn from(err: SdJwtRsError) -> SdjwtVerifierResultError {
        SdjwtVerifierResultError::SdJwtRsError(err.to_string())
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
    #[error("Unauthorised: expected {0}, got {1}")]
    Unauthorised(String, String),
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

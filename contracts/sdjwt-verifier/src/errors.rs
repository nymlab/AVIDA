use avida_cheqd::ibc::ChannelError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use serde_json_wasm::de::Error as SerdeJsonError;
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

#[derive(Error, Debug, PartialEq)]
pub enum SdjwtVerifierError {
    #[error("Verifier Result Error")]
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

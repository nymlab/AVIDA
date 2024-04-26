use cosmwasm_std::StdError;
use serde_json_wasm::de::Error as SerdeJsonError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SdjwtVerifierError {
    #[error("sdjwt {0}")]
    SdJwt(String),
    #[error("String Conversion {0}")]
    StringConversion(String),
    #[error("Jwt Conversion {0}")]
    JwtError(String),
    #[error("Serde JSON Error")]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("data deserialization error")]
    DataDeserialization,
    #[error("base64 decode error")]
    Base64DecodeError,
    #[error("Presentation Too Large")]
    PresentationTooLarge,
    #[error("Verified Claims should be an Object Map")]
    VerifiedClaimsTypeUnexpected,
    #[error("Criterion Value Type Unexpected")]
    CriterionValueTypeUnexpected,
    #[error("Criterion Value Number Unexpected")]
    CriterionValueNumberInvalid,
    #[error("No Disclosed Claims {0}")]
    DisclosedClaimNotFound(String),
    #[error("App Already Registered")]
    AppAlreadyRegistered,
    #[error("Unauthorised")]
    Unauthorised,
    #[error("Unsupported Key Type")]
    UnsupportedKeyType,
    #[error("Route Not Registered")]
    RouteNotRegistered,
    #[error("Required Claims Not Satisfied")]
    RequiredClaimsNotSatisfied,
    #[error("PubKey Not Found")]
    PubKeyNotFound,
}

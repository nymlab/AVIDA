use avida_cheqd::ibc::ChannelError;
use cosmwasm_std::{Instantiate2AddressError, StdError};
use serde_json_wasm::de::Error as SerdeJsonError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SdjwtVerifierError {
    #[error("IBC returned resource format unexpected {0}")]
    ReturnedResourceFormat(String),
    #[error("IBC channel already exists")]
    ChannelAlreadyExists,
    #[error("Serde JSON Error")]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Instantiate2(#[from] Instantiate2AddressError),
    #[error("data deserialization error")]
    DataDeserialization,
    #[error("Presentation Too Large")]
    PresentationTooLarge,
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
    #[error("channel error")]
    ChannelError(#[from] ChannelError),
    #[error("Route Requirement Error")]
    RouteRequirementError,
    #[error("PubKey Not Found")]
    PubKeyNotFound,
}

impl From<SdjwtVerifierError> for StdError {
    fn from(err: SdjwtVerifierError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}

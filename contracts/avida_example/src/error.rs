use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Parse Reply Error")]
    ParseReplyError,

    #[error("Registration Error {0}")]
    RegistrationError(String),

    #[error("Verification Process Error {0}")]
    VerificationProcessError(String),

    #[error("Verification Error {0}")]
    VerificationError(String),

    #[error("Get Route Requirements Error {0}")]
    GetRouteRequirementsError(String),

    #[error("Invalid RouteId")]
    InvalidRouteId,
}

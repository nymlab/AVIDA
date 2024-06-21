use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Parse Reply Error {0}")]
    ParseReplyError(#[from] ParseReplyError),

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

impl From<ContractError> for cosmwasm_std::StdError {
    fn from(e: ContractError) -> cosmwasm_std::StdError {
        cosmwasm_std::StdError::generic_err(format!("{}", e))
    }
}

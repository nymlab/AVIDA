use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ParseReplyError(#[from] ParseReplyError),

    #[error("{0}")]
    RegistrationError(String),

    #[error("{0}")]
    VerificationProcessError(String),

    #[error("{0}")]
    GetRouteRequirementsError(String),

    #[error("InvalidRouteId")]
    InvalidRouteId,
}

impl From<ContractError> for cosmwasm_std::StdError {
    fn from(e: ContractError) -> cosmwasm_std::StdError {
        cosmwasm_std::StdError::generic_err(format!("{}", e))
    }
}
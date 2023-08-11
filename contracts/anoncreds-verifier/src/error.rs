use crate::helpers::ibc::ChannelError;
use cosmwasm_std::StdError;
use ssi::traits::resource_over_ibc_interface::ResourceOverIbcError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("IBC: {0}")]
    ResourceOverIbcError(#[from] ResourceOverIbcError),

    #[error("Channel: {0}")]
    ChannelError(#[from] ChannelError),
}

pub use crate::consts::{APP_ORDER, CHEQD_APP_PORT_ID, IBC_APP_VERSION, PACKET_LIFETIME};
use cosmwasm_std::{Env, IbcChannel, IbcOrder, Timestamp};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ChannelError {
    #[error("Only supports unordered channels")]
    InvalidChannelOrder,

    #[error("Counterparty version must be '{0}'")]
    InvalidChannelVersion(&'static str),

    #[error("Only supports cheqd port")]
    InvalidPort,
}

pub fn check_order(order: &IbcOrder) -> Result<(), ChannelError> {
    if order != &APP_ORDER {
        Err(ChannelError::InvalidChannelOrder)
    } else {
        Ok(())
    }
}

pub fn check_version(version: &str) -> Result<(), ChannelError> {
    if version != IBC_APP_VERSION {
        Err(ChannelError::InvalidChannelVersion(IBC_APP_VERSION))
    } else {
        Ok(())
    }
}

pub fn check_app_port(ibc_channel: &IbcChannel) -> Result<(), ChannelError> {
    if ibc_channel.counterparty_endpoint.port_id != CHEQD_APP_PORT_ID {
        Err(ChannelError::InvalidPort)
    } else {
        Ok(())
    }
}

pub fn get_timeout_timestamp(env: &Env) -> Timestamp {
    env.block.time.plus_seconds(PACKET_LIFETIME)
}

use cosmwasm_std::{
    from_json, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcOrder, IbcPacketAckMsg, StdAck, StdError, StdResult, Timestamp,
};

use crate::types::{ResourceReqPacket, ResourceWithMetadata};
use thiserror::Error;

/// This is the same as the cheqd resource module IBC version
pub const IBC_APP_VERSION: &str = "cheqd-resource-v3";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const CHEQD_APP_PORT_ID: &str = "cheqdresource";
pub const HOUR_PACKET_LIFETIME: u64 = 60 * 60; // in seconds
                                               //
#[derive(Error, Debug, PartialEq)]
pub enum ChannelError {
    #[error("Only supports unordered channels")]
    InvalidChannelOrder,

    #[error("Counterparty version must be '{0}'")]
    InvalidChannelVersion(&'static str),

    #[error("Only supports cheqd port")]
    InvalidPort,
}

impl From<ChannelError> for StdError {
    fn from(err: ChannelError) -> StdError {
        StdError::generic_err(err.to_string())
    }
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

pub fn get_timeout_timestamp(env: &Env, timeout: u64) -> Timestamp {
    env.block.time.plus_seconds(timeout)
}
/// Handler for channel opening,
/// enforces ordering and versioning constraints
pub fn ibc_channel_open_handler(msg: IbcChannelOpenMsg) -> StdResult<IbcChannelOpenResponse> {
    let channel = msg.channel();

    // check ordering
    check_order(&channel.order)?;

    // In ibcv3 we don't check the version string passed in the message
    // and only check the counterparty version.
    // This contract is targetd to connect with cheqd resource module
    // ibc_app_version = cheqd-resource-v3
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    // Counterparty info cannot be trusted, but we are adding this check anyway
    check_app_port(msg.channel())?;

    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

/// Error on closing channel
pub fn ibc_channel_close_handler() -> StdResult<IbcBasicResponse> {
    Err(StdError::generic_err("Closing is not supported"))
}

/// Checks ack resource matching requested resource
pub fn ibc_packet_ack_resource_extractor(
    msg: IbcPacketAckMsg,
) -> StdResult<(ResourceReqPacket, ResourceWithMetadata)> {
    let ack: StdAck = from_json(&msg.acknowledgement.data)?;
    match ack {
        StdAck::Success(binary) => {
            let resource: ResourceWithMetadata = from_json(&binary)?;
            let original_packet: ResourceReqPacket = from_json(&msg.original_packet.data)?;

            if original_packet.resource_id != resource.linked_resource_metadata.resource_id {
                Err(StdError::generic_err("Ack Returned Unmatched resource_id"))
            } else if original_packet.collection_id
                != resource.linked_resource_metadata.resource_collection_id
            {
                Err(StdError::generic_err(
                    "Ack Returned Unmatched collection_id",
                ))
            } else {
                Ok((original_packet, resource))
            }
        }
        StdAck::Error(err) => Err(StdError::generic_err(format!("Ack Returned Err: {}", err))),
    }
}

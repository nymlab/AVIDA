use crate::{
    consts::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
    ibc::{ibc_channel_connect_handler, ibc_channel_open_handler, ibc_packet_ack_handler},
};

use cosmwasm_std::{
    entry_point, DepsMut, Env, Event, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use sylvia::{
    contract, schemars,
    types::{InstantiateCtx, QueryCtx},
};

use ssi::{
    traits::resource_over_ibc_interface,
    types::{ResourceWithMetadata, StdAck},
};

/// The main strcut for the contract
pub struct AnonCredsVerifier<'a> {
    pub channel: Item<'a, IbcChannel>,
    /// Storage of resources and their last update
    // Map of (resource_id, collection_id) to the actual resource
    // TODO: snapshot or other ways to track updates if helpful?
    pub resources: Map<'a, (&'a str, &'a str), ResourceWithMetadata>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(ContractError)]
#[messages(resource_over_ibc_interface as ResourceOverIbcInterface)]
impl<'a> AnonCredsVerifier<'a> {
    pub const fn new() -> Self {
        Self {
            channel: Item::new("channel"),
            resources: Map::new("resources"),
        }
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, ContractError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        let event = Event::new("vectis.AnonCredsVerifier.v1");
        Ok(Response::new().add_event(event))
    }

    #[msg(query)]
    fn channel(&self, ctx: QueryCtx) -> Result<Option<IbcChannel>, StdError> {
        self.channel.may_load(ctx.deps.storage)
    }

    fn ibc_channel_connect(
        &self,
        deps: DepsMut,
        msg: IbcChannelConnectMsg,
    ) -> Result<, ContractError> {
    }
}

#[entry_point]
/// The entry point for opening a channel
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    ibc_channel_open_handler(msg)
}
#[entry_point]
/// The entry point for connecting a channel
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    ibc_channel_connect_handler(deps, msg)
}
#[entry_point]
/// The entry point for connecting a channel
// NOTE: to be moved when implemented by sylvia
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Err(StdError::generic_err("Closing is not supported"))
}

#[entry_point]
/// This should never be used as we do not have services over IBC (at the moment)
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    (|| Ok(IbcReceiveResponse::new().set_ack(StdAck::fail(format!("No packet handling")))))()
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    ibc_packet_ack_handler(deps, env, msg)
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

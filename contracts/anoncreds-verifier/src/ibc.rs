use crate::{
    consts::IBC_APP_VERSION,
    contract::AnonCredsVerifier,
    error::ContractError,
    helpers::{
        events::new_event,
        ibc::{check_app_port, check_order, check_version, get_timeout_timestamp},
    },
};

use cosmwasm_std::{
    from_binary, to_binary, CosmosMsg, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg, IbcTimeout,
    Response, StdError, StdResult, SubMsg,
};

use sylvia::{
    contract,
    types::{ExecCtx, QueryCtx},
};

use ssi::{
    traits::{resource_over_ibc_interface, resource_over_ibc_interface::ResourceOverIbcInterface},
    types::{ResourceReqPacket, ResourceWithMetadata, StdAck},
};

#[contract(module=crate::contract)]
#[messages(resource_over_ibc_interface as ResourceOverIbcInterface)]
impl<'a> ResourceOverIbcInterface for AnonCredsVerifier<'a> {
    type Error = ContractError;

    #[msg(exec)]
    fn update_state(
        &self,
        ctx: ExecCtx,
        resource_id: String,
        collection_id: String,
    ) -> Result<Response, Self::Error> {
        let ibc_msg = SubMsg::new(CosmosMsg::Ibc(cosmwasm_std::IbcMsg::SendPacket {
            channel_id: self.channel.load(ctx.deps.storage)?.endpoint.channel_id,
            data: to_binary(&ResourceReqPacket {
                resource_id,
                collection_id,
            })?,
            timeout: IbcTimeout::with_timestamp(get_timeout_timestamp(&ctx.env)),
        }));
        Ok(Response::new().add_submessage(ibc_msg))
    }

    #[msg(query)]
    fn query_state(
        &self,
        ctx: QueryCtx,
        resource_id: String,
        collection_id: String,
    ) -> Result<ResourceWithMetadata, Self::Error> {
        let res = self
            .resources
            .load(ctx.deps.storage, (&resource_id, &collection_id))?;
        Ok(res)
    }
}

/// Handler for channel opening,
/// enforces ordering and versioning constraints
pub fn ibc_channel_open_handler(
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
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

/// We expect a one channel connection with cheqd.
/// We do not allow channels to be closed.
/// On initial deployment of this contract we will create one channel and store that.
/// In the future we can make this updatable.
pub fn ibc_channel_connect_handler(
    deps: DepsMut,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let contract = AnonCredsVerifier::new();
    if let Some(_) = contract.channel.may_load(deps.storage)? {
        Err(StdError::generic_err("Channel already exist"))
    } else {
        contract.channel.save(deps.storage, &msg.channel())?;

        let event = new_event()
            .add_attribute("action", "ibc channel connection")
            .add_attribute("channel_id", &msg.channel().endpoint.channel_id);

        Ok(IbcBasicResponse::new().add_event(event))
    }
}

/// This handles the storing of resources
pub fn ibc_packet_ack_handler(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let ack: StdAck = from_binary(&msg.acknowledgement.data)?;
    match ack {
        StdAck::Result(binary) => {
            let resource: ResourceWithMetadata = from_binary(&binary)?;
            let original_packet: ResourceReqPacket = from_binary(&msg.original_packet.data)?;

            if original_packet.resource_id != resource.linked_resource_metadata.resource_id {
                Err(StdError::generic_err("Ack Returned Unmatched resource_id"))
            } else if original_packet.collection_id
                != resource.linked_resource_metadata.resource_collection_id
            {
                Err(StdError::generic_err(
                    "Ack Returned Unmatched collection_id",
                ))
            } else {
                let contract = AnonCredsVerifier::new();

                contract.resources.save(
                    deps.storage,
                    (&original_packet.resource_id, &original_packet.collection_id),
                    &resource,
                )?;
                let event = new_event()
                    .add_attribute("action", "Ack")
                    .add_attribute("resource_id", original_packet.resource_id)
                    .add_attribute("collection_id", original_packet.collection_id);

                Ok(IbcBasicResponse::new().add_event(event))
            }
        }
        StdAck::Error(err) => Err(StdError::generic_err(format!("Ack Returned Err: {}", err))),
    }
    // First we check cheqd returned the requested data
}

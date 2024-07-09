use crate::{
    errors::SdjwtVerifierError,
    types::{
        Criterion, InitRegistration, PendingRoute, UpdateRevocationListRequest,
        VerificationRequirements, IDX,
    },
};

// AVIDA specific
use avida_cheqd::ibc::{
    ibc_channel_close_handler, ibc_channel_open_handler, ibc_packet_ack_resource_extractor,
};
use avida_common::{
    traits::avida_verifier_trait,
    types::{IssuerSourceOrData, MaxPresentationLen, RouteId, MAX_PRESENTATION_LEN},
};
//  CosmWasm / Sylvia lib
use cosmwasm_std::{
    entry_point, from_json, Addr, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Response, StdAck, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};
#[cfg(not(feature = "library"))]
use sylvia::entry_points;
use sylvia::{
    contract, schemars,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};

use jsonwebtoken::jwk::Jwk;
use std::collections::HashMap;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The `invoice factory` structure stored in state
pub struct SdjwtVerifier<'a> {
    /// Max Presentation Length
    pub max_presentation_len: MaxPresentationLen<'a>,
    /// Registered Smart Contract addrs and routes
    pub app_trust_data_source: Map<'a, &'a str, HashMap<RouteId, IssuerSourceOrData>>,
    /// Per route, the requirements that is required for verifier to make a decision
    /// This contains the presentation required (i.e. disclosed value requirements) and the issuer
    /// pubkey
    pub app_routes_requirements: Map<'a, &'a str, HashMap<RouteId, VerificationRequirements>>,
    /// Registered Smart Contract addrs and their admins
    pub app_admins: Map<'a, &'a str, Addr>,
    /// The IBC channel connecting with cheqd resource
    pub channel_id: Item<'a, String>,
    /// Temp storage pending IBC packet Ack
    /// ibc_channel_ack: the original packet is a ResourceReqPacket which should fill the `VerificationReq`
    /// for a app and its route.
    /// NOTE: There is currently no clean up / expiration in this version
    /// so we will only support one per packet at the moment (and it will be overwritten)
    pub pending_verification_req_requests: Map<'a, &'a str, PendingRoute>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::error(SdjwtVerifierError)]
#[sv::messages(avida_verifier_trait as AvidaVerifierTrait)]
impl SdjwtVerifier<'_> {
    pub fn new() -> Self {
        Self {
            max_presentation_len: MAX_PRESENTATION_LEN,
            app_trust_data_source: Map::new("data_sources"),
            app_routes_requirements: Map::new("routes_requirements"),
            app_admins: Map::new("admins"),
            channel_id: Item::new("channel_id"),
            pending_verification_req_requests: Map::new("pending_verification_req_requests"),
        }
    }

    /// Instantiates sdjwt verifier
    #[sv::msg(instantiate)]
    fn instantiate(
        &self,
        ctx: InstantiateCtx,
        max_presentation_len: usize,
        // Vec of app_addr to their routes and requirements
        init_registrations: Vec<InitRegistration>,
    ) -> Result<Response, SdjwtVerifierError> {
        let InstantiateCtx { deps, env, .. } = ctx;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        self.max_presentation_len
            .save(deps.storage, &max_presentation_len)?;

        for app in init_registrations {
            let admin = deps.api.addr_validate(&app.app_admin)?;
            let app_addr = deps.api.addr_validate(&app.app_addr)?;
            self._register(deps.storage, &env, &admin, app_addr.as_str(), app.routes)?;
        }

        Ok(Response::default())
    }

    #[sv::msg(exec)]
    fn update_revocation_list(
        &self,
        ctx: ExecCtx,
        app_addr: String,
        request: UpdateRevocationListRequest,
    ) -> Result<Response, SdjwtVerifierError> {
        let UpdateRevocationListRequest {
            route_id,
            revoke,
            unrevoke,
        } = request;

        let mut all_routes_requirements = self
            .app_routes_requirements
            .load(ctx.deps.storage, &app_addr)?;

        let mut route_requirements = all_routes_requirements
            .get(&route_id)
            .ok_or(SdjwtVerifierError::RouteNotRegistered)?
            .clone();

        route_requirements
            .presentation_required
            .iter_mut()
            .find(|(key, _)| key == IDX)
            .map(|(_, criterion)| -> Result<_, SdjwtVerifierError> {
                if let Criterion::NotContainedIn(revocation_list) = criterion {
                    for r in revoke {
                        if !revocation_list.contains(&r) {
                            revocation_list.push(r);
                        }
                    }

                    for r in unrevoke {
                        revocation_list.retain(|&x| x != r);
                    }
                    Ok(())
                } else {
                    Err(SdjwtVerifierError::RevocationListType)
                }
            })
            .ok_or(SdjwtVerifierError::IDXNotInRequirement)??;

        all_routes_requirements.insert(route_id, route_requirements);

        self.app_routes_requirements
            .save(ctx.deps.storage, &app_addr, &all_routes_requirements)?;

        Ok(Response::default())
    }

    #[sv::msg(query)]
    fn get_route_verification_key(
        &self,
        ctx: QueryCtx,
        app_addr: String,
        route_id: RouteId,
    ) -> Result<Option<String>, SdjwtVerifierError> {
        let req = self
            .app_routes_requirements
            .load(ctx.deps.storage, &app_addr)?;
        let route_req = req
            .get(&route_id)
            .ok_or(SdjwtVerifierError::RouteNotRegistered)?;
        Ok(route_req
            .issuer_pubkey
            .as_ref()
            .map(|jwk| serde_json_wasm::to_string(jwk).unwrap()))
    }

    // Functions in the `impl` block has access to the state of the contract
    fn ibc_channel_connect(
        &self,
        deps: DepsMut,
        msg: IbcChannelConnectMsg,
    ) -> Result<IbcBasicResponse, SdjwtVerifierError> {
        if self.channel_id.may_load(deps.storage)?.is_some() {
            Err(SdjwtVerifierError::ChannelAlreadyExists)
        } else {
            self.channel_id
                .save(deps.storage, &msg.channel().endpoint.channel_id)?;

            Ok(IbcBasicResponse::new())
        }
    }

    fn ibc_packet_ack(
        &self,
        deps: DepsMut,
        msg: IbcPacketAckMsg,
    ) -> Result<IbcBasicResponse, SdjwtVerifierError> {
        let (resource_req_packet, resource) = ibc_packet_ack_resource_extractor(msg)?;

        // Checks that this was a packet that we requested
        let contract = SdjwtVerifier::new();
        let pending_route = contract
            .pending_verification_req_requests
            .load(deps.storage, &resource_req_packet.to_string())?;
        contract
            .pending_verification_req_requests
            .remove(deps.storage, &resource_req_packet.to_string());

        // Checks the return data is the expected format
        let pubkey: Jwk = from_json(resource.linked_resource.data)
            .map_err(|e| SdjwtVerifierError::ReturnedResourceFormat(e.to_string()))?;

        let mut req = contract
            .app_routes_requirements
            .load(deps.storage, &pending_route.app_addr)?;

        let r = req
            .get_mut(&pending_route.route_id)
            .ok_or(SdjwtVerifierError::NoRequirementsForRoute)?;

        r.issuer_pubkey = Some(pubkey);

        contract
            .app_routes_requirements
            .save(deps.storage, &pending_route.app_addr, &req)?;

        Ok(IbcBasicResponse::new())
    }
}

impl Default for SdjwtVerifier<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[entry_point]
/// The entry point for connecting a channel
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(SdjwtVerifier::new().ibc_channel_connect(deps, msg)?)
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(SdjwtVerifier::new().ibc_packet_ack(deps, msg)?)
}

#[entry_point]
/// The entry point for opening a channel
// NOTE: to be moved when implemented by sylvia
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, SdjwtVerifierError> {
    Ok(ibc_channel_open_handler(msg)?)
}

#[entry_point]
/// The entry point for connecting a channel
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    // Returns error as it does not support closing
    ibc_channel_close_handler()
}

#[entry_point]
/// This should never be used as we do not have services over IBC (at the moment)
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, SdjwtVerifierError> {
    Ok(IbcReceiveResponse::new().set_ack(StdAck::error("No packet handling".to_string())))
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

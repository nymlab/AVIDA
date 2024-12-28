#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo, Response, StdAck,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};

use crate::{
    errors::SdjwtVerifierError,
    types::{
        Criterion, InitRegistration, PendingRoute, UpdateRevocationListRequest,
        VerificationRequirements, IDX,
    },
};

use avida_cheqd::ibc::{
    ibc_channel_close_handler, ibc_channel_open_handler, ibc_packet_ack_resource_extractor,
};
use avida_common::types::{
    IssuerSourceOrData, MaxPresentationLen, RegisterRouteRequest, RouteId,
    RouteVerificationRequirements, VerfiablePresentation, MAX_PRESENTATION_LEN,
};

use jsonwebtoken::jwk::Jwk;
use std::collections::HashMap;

// Contract name and version info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// State structure
pub struct SdjwtVerifier<'a> {
    pub max_presentation_len: MaxPresentationLen,
    pub app_trust_data_source: Map<&'a str, HashMap<RouteId, IssuerSourceOrData>>,
    pub app_routes_requirements: Map<&'a str, HashMap<RouteId, VerificationRequirements>>,
    pub app_admins: Map<&'a str, Addr>,
    pub channel_id: Item<String>,
    pub pending_verification_req_requests: Map<&'a str, PendingRoute>,
}

// Contract instantiation parameters
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub max_presentation_len: usize,
    pub init_registrations: Vec<InitRegistration>,
}

// Execute messages
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    UpdateRevocationList {
        app_addr: String,
        request: UpdateRevocationListRequest,
    },
    Register {
        app_addr: String,
        requests: Vec<RegisterRouteRequest>,
    },
    Verify {
        presentation: VerfiablePresentation,
        route_id: RouteId,
        app_addr: Option<String>,
        additional_requirements: Option<Binary>,
    },
    Update {
        app_addr: String,
        route_id: RouteId,
        route_criteria: Option<RouteVerificationRequirements>,
    },
    Deregister {
        app_addr: String,
    },
}

// Query messages
#[cosmwasm_schema::cw_serde]
pub enum QueryMsg {
    GetRouteVerificationKey { app_addr: String, route_id: RouteId },
    GetAppAdmin { app_addr: String },
    GetRoutes { app_addr: String },
    GetRouteRequirements { app_addr: String, route_id: RouteId },
}

// Sudo messages (privileged operations)
#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    Verify {
        app_addr: String,
        route_id: RouteId,
        presentation: VerfiablePresentation,
        additional_requirements: Option<Binary>,
    },
    Update {
        app_addr: String,
        route_id: RouteId,
        route_criteria: Option<RouteVerificationRequirements>,
    },
    Register {
        app_addr: String,
        app_admin: String,
        routes: Vec<RegisterRouteRequest>,
    },
}

impl<'a> Default for SdjwtVerifier<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SdjwtVerifier<'a> {
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
}

// Entry points
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, SdjwtVerifierError> {
    let contract = SdjwtVerifier::new();
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    contract
        .max_presentation_len
        .save(deps.storage, &msg.max_presentation_len)?;

    for app in msg.init_registrations {
        let admin = deps.api.addr_validate(&app.app_admin)?;
        let app_addr = deps.api.addr_validate(&app.app_addr)?;
        contract._register(deps.storage, &env, &admin, app_addr.as_str(), app.routes)?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, SdjwtVerifierError> {
    let contract = SdjwtVerifier::new();
    match msg {
        ExecuteMsg::UpdateRevocationList { app_addr, request } => {
            contract.handle_update_revocation_list(deps, app_addr, request)
        }
        ExecuteMsg::Register { app_addr, requests } => {
            contract.handle_register(deps, env, info, app_addr, requests)
        }
        ExecuteMsg::Verify {
            presentation,
            route_id,
            app_addr,
            additional_requirements,
        } => contract.handle_verify(
            deps,
            env,
            info,
            presentation,
            route_id,
            app_addr,
            additional_requirements,
        ),
        ExecuteMsg::Update {
            app_addr,
            route_id,
            route_criteria,
        } => contract.handle_update(deps, env, info, app_addr, route_id, route_criteria),
        ExecuteMsg::Deregister { app_addr } => contract.handle_deregister(deps, info, app_addr),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let contract = SdjwtVerifier::new();
    match msg {
        QueryMsg::GetRouteVerificationKey { app_addr, route_id } => {
            to_json_binary(&contract.query_route_verification_key(deps, app_addr, route_id)?)
        }
        QueryMsg::GetAppAdmin { app_addr } => {
            to_json_binary(&contract.query_app_admin(deps, app_addr)?)
        }
        QueryMsg::GetRoutes { app_addr } => to_json_binary(&contract.query_routes(deps, app_addr)?),
        QueryMsg::GetRouteRequirements { app_addr, route_id } => {
            to_json_binary(&contract.query_route_requirements(deps, app_addr, route_id)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, SdjwtVerifierError> {
    let contract = SdjwtVerifier::new();
    match msg {
        SudoMsg::Verify {
            app_addr,
            route_id,
            presentation,
            additional_requirements,
        } => contract.handle_sudo_verify(
            deps,
            env,
            app_addr,
            route_id,
            presentation,
            additional_requirements,
        ),
        SudoMsg::Update {
            app_addr,
            route_id,
            route_criteria,
        } => contract.handle_sudo_update(deps, env, app_addr, route_id, route_criteria),
        SudoMsg::Register {
            app_addr,
            app_admin,
            routes,
        } => {
            let admin = deps.api.addr_validate(&app_admin)?;
            contract._register(deps.storage, &env, &admin, &app_addr, routes)
        }
    }
}

// IBC entry points
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, SdjwtVerifierError> {
    Ok(ibc_channel_open_handler(msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(SdjwtVerifier::new().ibc_channel_connect(deps, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    ibc_channel_close_handler()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, SdjwtVerifierError> {
    Ok(IbcReceiveResponse::new(StdAck::error("No packet handling".to_string())))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(SdjwtVerifier::new().ibc_packet_ack(deps, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

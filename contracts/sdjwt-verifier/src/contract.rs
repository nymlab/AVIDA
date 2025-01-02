use avida_common::types::AvidaVerifierExecuteMsg;
use avida_common::types::AvidaVerifierSudoMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo, Response, StdAck,
    StdResult,
};
use cw2::set_contract_version;

use crate::{
    errors::SdjwtVerifierError,
    msg::{InstantiateMsg, QueryMsg},
    state::MAX_PRESENTATION_LENGTH,
    verifier::*,
};

use avida_cheqd::ibc::{ibc_channel_close_handler, ibc_channel_open_handler};

// Contract name and version info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Entry points
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, SdjwtVerifierError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    MAX_PRESENTATION_LENGTH.save(deps.storage, &msg.max_presentation_len)?;

    for app in msg.init_registrations {
        let admin = deps.api.addr_validate(&app.app_admin)?;
        let app_addr = deps.api.addr_validate(&app.app_addr)?;
        _register(deps.storage, &env, &admin, app_addr.as_str(), app.routes)?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AvidaVerifierExecuteMsg,
) -> Result<Response, SdjwtVerifierError> {
    match msg {
        AvidaVerifierExecuteMsg::UpdateRevocationList { app_addr, request } => {
            handle_update_revocation_list(deps, app_addr, request)
        }
        AvidaVerifierExecuteMsg::Register { app_addr, requests } => {
            handle_register(deps, env, info, app_addr, requests)
        }
        AvidaVerifierExecuteMsg::Verify {
            presentation,
            route_id,
            app_addr,
            additional_requirements,
        } => handle_verify(
            deps,
            env,
            info,
            presentation,
            route_id,
            app_addr,
            additional_requirements,
        ),
        AvidaVerifierExecuteMsg::Update {
            app_addr,
            route_id,
            route_criteria,
        } => handle_update(deps, env, info, app_addr, route_id, route_criteria),
        AvidaVerifierExecuteMsg::Deregister { app_addr } => handle_deregister(deps, info, app_addr),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRouteVerificationKey { app_addr, route_id } => {
            let route_verification_key = query_route_verification_key(deps, app_addr, route_id)?;
            to_json_binary(&route_verification_key)
        }
        QueryMsg::GetAppAdmin { app_addr } => {
            let app_admin = query_app_admin(deps, app_addr)?;
            to_json_binary(&app_admin)
        }
        QueryMsg::GetRoutes { app_addr } => {
            let routes = query_routes(deps, app_addr)?;
            to_json_binary(&routes)
        }
        QueryMsg::GetRouteRequirements { app_addr, route_id } => {
            let route_requirements = query_route_requirements(deps, app_addr, route_id)?;
            to_json_binary(&route_requirements)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: AvidaVerifierSudoMsg,
) -> Result<Response, SdjwtVerifierError> {
    match msg {
        AvidaVerifierSudoMsg::Verify {
            app_addr,
            route_id,
            presentation,
            additional_requirements,
        } => handle_sudo_verify(
            deps,
            env,
            app_addr,
            route_id,
            presentation,
            additional_requirements,
        ),
        AvidaVerifierSudoMsg::Update {
            app_addr,
            route_id,
            route_criteria,
        } => handle_sudo_update(deps, env, app_addr, route_id, route_criteria),
        AvidaVerifierSudoMsg::Register {
            app_addr,
            app_admin,
            routes,
        } => {
            let admin = deps.api.addr_validate(&app_admin)?;
            _register(deps.storage, &env, &admin, &app_addr, routes)
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
    Ok(ibc_channel_connect_handler(deps, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    ibc_channel_close_handler()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, SdjwtVerifierError> {
    Ok(IbcReceiveResponse::new(StdAck::error(
        "No packet handling".to_string(),
    )))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(ibc_packet_ack_handler(deps, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

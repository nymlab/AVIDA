use crate::{
    errors::{SdjwtVerifierError, SdjwtVerifierResultError},
    state::*,
    types::{
        validate, Criterion, JwkInfo, PendingRoute, PresentationReq, VerificationRequirements,
        VerifyResult, _RegistrationRequest, IDX, ISS_KEY,
    },
};
use avida_cheqd::{
    ibc::{get_timeout_timestamp, ibc_packet_ack_resource_extractor, HOUR_PACKET_LIFETIME},
    types::ResourceReqPacket,
};
use avida_common::types::{
    IssuerSourceOrData, RegisterRouteRequest, RouteId, RouteVerificationRequirements,
    TrustRegistry, VerfiablePresentation, MAX_PRESENTATION_LENGTH,
};
use cosmwasm_std::Order;
use cosmwasm_std::{
    ensure, from_json, to_json_binary, Addr, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env,
    IbcBasicResponse, IbcChannelConnectMsg, IbcPacketAckMsg, IbcTimeout, MessageInfo, Response,
    Storage, SubMsg,
};
use sd_jwt_rs::{SDJWTSerializationFormat, SDJWTVerifier};
use serde_json::Value;
use std::collections::HashMap;

use avida_common::types::UpdateRevocationListRequest;
use jsonwebtoken::{
    jwk::{AlgorithmParameters, EllipticCurve, Jwk, OctetKeyPairParameters},
    DecodingKey,
};
use sd_jwt_rs::SDJWTCommon;

// Execute message handlers
pub fn handle_update_revocation_list(
    deps: DepsMut,
    info: MessageInfo,
    app_addr: String,
    request: UpdateRevocationListRequest,
) -> Result<Response, SdjwtVerifierError> {
    let UpdateRevocationListRequest {
        route_id,
        revoke,
        unrevoke,
    } = request;

    let mut route_requirements =
        APP_ROUTES_REQUIREMENTS.load(deps.storage, (app_addr.clone(), route_id))?;

    let valid_app_addr = deps.api.addr_validate(&app_addr)?;

    let app_admin = APP_ADMINS
        .load(deps.storage, valid_app_addr.as_str())
        .map_err(|_| SdjwtVerifierError::AppIsNotRegistered)?;

    if app_admin != info.sender {
        return Err(SdjwtVerifierError::Unauthorised);
    }

    route_requirements
        .presentation_required
        .iter_mut()
        .find(|req| req.attribute == IDX)
        .map(|req| -> Result<_, SdjwtVerifierError> {
            if let Criterion::NotContainedIn(revocation_list) = &mut req.criterion {
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

    APP_ROUTES_REQUIREMENTS.save(deps.storage, (app_addr, route_id), &route_requirements)?;

    Ok(Response::default())
}

pub fn handle_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app_addr: String,
    requests: Vec<RegisterRouteRequest>,
) -> Result<Response, SdjwtVerifierError> {
    let app_addr = deps.api.addr_validate(&app_addr)?;
    _register(
        deps.storage,
        &env,
        &info.sender,
        app_addr.as_str(),
        requests,
    )
}

pub fn handle_verify(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    presentation: VerfiablePresentation,
    route_id: RouteId,
    app_addr: Option<String>,
    additional_requirements: Option<Binary>,
) -> Result<Response, SdjwtVerifierError> {
    let additional_requirements: Option<PresentationReq> =
        additional_requirements.map(from_json).transpose()?;
    let app_addr = app_addr.unwrap_or_else(|| info.sender.to_string());

    let requirements = APP_ROUTES_REQUIREMENTS
        .load(deps.storage, (app_addr, route_id))
        .map_err(|_| SdjwtVerifierError::RouteNotRegistered)?;

    let max_len = MAX_PRESENTATION_LENGTH.load(deps.storage)?;

    let res = _verify(
        presentation,
        requirements,
        max_len,
        &env.block,
        additional_requirements,
    );

    let data = to_json_binary(&VerifyResult { result: res })?;
    Ok(Response::default().set_data(data))
}

pub fn handle_update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app_addr: String,
    route_id: RouteId,
    route_criteria: Option<RouteVerificationRequirements>,
) -> Result<Response, SdjwtVerifierError> {
    let app_addr = deps.api.addr_validate(&app_addr)?;

    let app_admin = APP_ADMINS
        .load(deps.storage, app_addr.as_str())
        .map_err(|_| SdjwtVerifierError::AppIsNotRegistered)?;

    if app_admin != info.sender {
        return Err(SdjwtVerifierError::Unauthorised);
    }

    _update(
        deps.storage,
        &env,
        app_addr.as_str(),
        route_id,
        route_criteria,
    )
}

pub fn handle_deregister(
    deps: DepsMut,
    info: MessageInfo,
    app_addr: String,
) -> Result<Response, SdjwtVerifierError> {
    if APP_ROUTES_REQUIREMENTS
        .prefix(app_addr.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .next()
        .is_none()
    {
        return Err(SdjwtVerifierError::AppIsNotRegistered);
    }

    let app_addr = deps.api.addr_validate(&app_addr)?;
    let app_admin = APP_ADMINS.load(deps.storage, app_addr.as_str())?;

    if app_admin != info.sender {
        return Err(SdjwtVerifierError::Unauthorised);
    }

    _deregister(deps.storage, app_addr.as_str())
}

// Sudo message handlers
pub fn handle_sudo_verify(
    deps: DepsMut,
    env: Env,
    app_addr: String,
    route_id: RouteId,
    presentation: VerfiablePresentation,
    additional_requirements: Option<Binary>,
) -> Result<Response, SdjwtVerifierError> {
    let additional_requirements: Option<PresentationReq> =
        additional_requirements.map(from_json).transpose()?;

    let requirements = APP_ROUTES_REQUIREMENTS
        .load(deps.storage, (app_addr, route_id))
        .map_err(|_| SdjwtVerifierError::RouteNotRegistered)?
        .clone();
    let max_len = MAX_PRESENTATION_LENGTH.load(deps.storage)?;

    let res = _verify(
        presentation,
        requirements,
        max_len,
        &env.block,
        additional_requirements,
    )
    .map(|res| to_json_binary(&VerifyResult { result: Ok(res) }))
    .map_err(SdjwtVerifierError::SdjwtVerifierResultError)??;

    Ok(Response::default().set_data(res))
}

pub fn handle_sudo_update(
    deps: DepsMut,
    env: Env,
    app_addr: String,
    route_id: RouteId,
    route_criteria: Option<RouteVerificationRequirements>,
) -> Result<Response, SdjwtVerifierError> {
    _update(deps.storage, &env, &app_addr, route_id, route_criteria)
}

// Query handlers
pub fn query_route_verification_keys(
    deps: Deps,
    app_addr: String,
    route_id: RouteId,
) -> Result<Option<Vec<String>>, SdjwtVerifierError> {
    let route_req = APP_ROUTES_REQUIREMENTS
        .load(deps.storage, (app_addr, route_id))
        .map_err(|_| SdjwtVerifierError::RouteNotRegistered)?;

    let keys = route_req.issuer_pubkeys.as_ref().map(|jwks| {
        jwks.iter()
            .map(|(_, jwk)| serde_json::to_string(jwk).unwrap())
            .collect()
    });

    Ok(keys)
}

pub fn query_app_admin(deps: Deps, app_addr: String) -> Result<String, SdjwtVerifierError> {
    let admin = APP_ADMINS.load(deps.storage, &app_addr)?;
    Ok(admin.to_string())
}

pub fn query_routes(deps: Deps, app_addr: String) -> Result<Vec<RouteId>, SdjwtVerifierError> {
    let routes: Vec<RouteId> = APP_ROUTES_REQUIREMENTS
        .prefix(app_addr)
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(routes)
}

pub fn query_route_requirements(
    deps: Deps,
    app_addr: String,
    route_id: RouteId,
) -> Result<VerificationRequirements, SdjwtVerifierError> {
    let req = APP_ROUTES_REQUIREMENTS
        .load(deps.storage, (app_addr, route_id))
        .map_err(|_| SdjwtVerifierError::RouteNotRegistered)?;
    Ok(req)
}

/// Verify the provided presentation within the context of the given route
pub fn _verify(
    presentation: VerfiablePresentation,
    requirements: VerificationRequirements,
    max_presentation_len: usize,
    block_info: &BlockInfo,
    additional_requirements: Option<PresentationReq>,
) -> Result<Value, SdjwtVerifierResultError> {
    // Ensure the presentation is not too large
    ensure!(
        presentation.len() <= max_presentation_len,
        SdjwtVerifierResultError::PresentationTooLarge
    );

    let mut common = SDJWTCommon::default();
    common.parse_compact_sd_jwt(
        String::from_utf8(presentation.to_vec())
            .map_err(|e| SdjwtVerifierResultError::StringConversion(e.to_string()))?,
    )?;

    let payload = common
        .unverified_input_sd_jwt_payload
        .ok_or(SdjwtVerifierResultError::IssuerNotFound)?;

    let iss = payload
        .get(ISS_KEY)
        .ok_or(SdjwtVerifierResultError::IssuerNotFound)?
        .as_str()
        .ok_or(SdjwtVerifierResultError::StringConversion(
            "Iss is not a string".to_owned(),
        ))?;

    if let Some(pubkeys) = requirements.issuer_pubkeys {
        let decoding_key = if let Some(pubkey) = pubkeys.get(iss) {
            DecodingKey::from_jwk(pubkey)
                .map_err(|e| SdjwtVerifierResultError::JwtError(e.to_string()))?
        } else {
            return Err(SdjwtVerifierResultError::PubKeyNotFound);
        };

        // We verify the presentation
        let sdjwt_verifier = SDJWTVerifier::new(
            String::from_utf8(presentation.to_vec())
                .map_err(|e| SdjwtVerifierResultError::StringConversion(e.to_string()))?,
            Box::new(move |_, _| decoding_key.clone()),
            None, // This version does not support key binding
            None, // This version does not support key binding
            SDJWTSerializationFormat::Compact,
        )
        .map_err(|e| SdjwtVerifierResultError::SdJwt(e.to_string()))?;

        let combined_requirements = if let Some(additional_requirements) = additional_requirements {
            let mut combined_requirements = requirements.presentation_required.clone();
            combined_requirements.extend(additional_requirements);
            combined_requirements
        } else {
            requirements.presentation_required
        };

        // We validate the verified claims against the requirements
        validate(
            combined_requirements,
            sdjwt_verifier.verified_claims.clone(),
            block_info,
        )?;
        Ok(sdjwt_verifier.verified_claims)
    }
    // If the issuer is not in the requirements, we return an error
    else {
        Err(SdjwtVerifierResultError::IssuerNotFound)
    }
}

/// Performs a registration of an application and all its routes
pub fn _register(
    storage: &mut dyn Storage,
    env: &Env,
    admin: &Addr,
    app_addr: &str,
    route_criteria: Vec<RegisterRouteRequest>,
) -> Result<Response, SdjwtVerifierError> {
    if APP_ROUTES_REQUIREMENTS
        .prefix(app_addr.to_owned())
        .range(storage, None, None, Order::Ascending)
        .next()
        .is_some()
    {
        return Err(SdjwtVerifierError::AppAlreadyRegistered);
    }

    // Probably do not need these
    let mut data_sources: HashMap<u64, Vec<IssuerSourceOrData>> = HashMap::new();

    let mut response = Response::default();

    for RegisterRouteRequest {
        route_id,
        requirements,
    } in route_criteria
    // here we get the iss to insert into the APP_ROUTES_REQUIREMENTS
    {
        data_sources.insert(route_id, requirements.issuer_source_or_data.clone());
        // On registration we check if the dApp has request for IBC data
        // Make a verification request for specified app addr and route id with a provided route criteria
        let _RegistrationRequest {
            verification_requirements,
            ibc_msgs,
        } = make_internal_registration_request(storage, env, app_addr, route_id, requirements)?;

        // Save the registered trust data sources and route requirements
        APP_ROUTES_REQUIREMENTS.save(
            storage,
            (app_addr.to_owned(), route_id),
            &verification_requirements,
        )?;

        if let Some(ibc_msgs) = ibc_msgs {
            response = response.add_submessages(ibc_msgs);
        }
    }

    APP_ADMINS.save(storage, app_addr, admin)?;

    Ok(response)
}

/// Performs a deregister of an application and all its routes
fn _deregister(storage: &mut dyn Storage, app_addr: &str) -> Result<Response, SdjwtVerifierError> {
    APP_ROUTES_REQUIREMENTS
        .prefix(app_addr.to_owned())
        .clear(storage, None);
    APP_ADMINS.remove(storage, app_addr);

    Ok(Response::default())
}

/// Performs an update on the verification requirements for a given app addr and route id with the new criteria
fn _update(
    storage: &mut dyn Storage,
    env: &Env,
    app_addr: &str,
    route_id: RouteId,
    route_criteria: Option<RouteVerificationRequirements>,
) -> Result<Response, SdjwtVerifierError> {
    // Ensure the app with this address is registered
    if APP_ROUTES_REQUIREMENTS
        .prefix(app_addr.to_owned())
        .range(storage, None, None, Order::Ascending)
        .next()
        .is_none()
    {
        return Err(SdjwtVerifierError::AppIsNotRegistered);
    }

    let mut response: Response = Response::default();

    // On registration we check if the dApp has request for IBC data
    if let Some(route_criteria) = route_criteria {
        // Make a verification request for specified app addr and route id with a provided route criteria
        let _RegistrationRequest {
            verification_requirements,
            ibc_msgs,
        } = make_internal_registration_request(storage, env, app_addr, route_id, route_criteria)?;

        APP_ROUTES_REQUIREMENTS.save(
            storage,
            (app_addr.to_owned(), route_id),
            &verification_requirements,
        )?;

        if let Some(ibc_msgs) = ibc_msgs {
            response = response.add_submessages(ibc_msgs);
        }

        Ok(response)
    } else {
        APP_ROUTES_REQUIREMENTS.remove(storage, (app_addr.to_owned(), route_id));

        if APP_ROUTES_REQUIREMENTS
            .prefix(app_addr.to_owned())
            .range(storage, None, None, Order::Ascending)
            .next()
            .is_none()
        {
            _deregister(storage, app_addr)
        } else {
            Ok(response)
        }
    }
}

/// Creates a _RegitrationRequest for specified app addr and route id and provided route criteria
fn make_internal_registration_request(
    storage: &mut dyn Storage,
    env: &Env,
    app_addr: &str,
    route_id: RouteId,
    route_criteria: RouteVerificationRequirements,
) -> Result<_RegistrationRequest, SdjwtVerifierError> {
    let mut ibc_submsgs: Vec<SubMsg> = Vec::new();
    let mut issuer_pubkeys: HashMap<String, Jwk> = HashMap::new();

    let mut vr = VerificationRequirements::new(route_criteria.presentation_required, None)?;

    for isd in route_criteria.issuer_source_or_data.iter() {
        if let Some(registry) = &isd.source {
            match registry {
                // We query this data via IBC
                TrustRegistry::Cheqd => {
                    // For Cheqd, the data is in the ResourceReqPacket
                    let resource_req_packat: ResourceReqPacket = from_json(&isd.data_or_location)?;

                    let ibc_msg = SubMsg::new(CosmosMsg::Ibc(cosmwasm_std::IbcMsg::SendPacket {
                        channel_id: CHANNEL_ID.load(storage)?,
                        data: to_json_binary(&resource_req_packat)?,
                        timeout: IbcTimeout::with_timestamp(get_timeout_timestamp(
                            env,
                            HOUR_PACKET_LIFETIME,
                        )),
                    }));

                    PENDING_VERIFICATION_REQ_REQUESTS.save(
                        storage,
                        &resource_req_packat.to_string(),
                        &PendingRoute {
                            app_addr: app_addr.to_string(),
                            route_id,
                        },
                    )?;

                    ibc_submsgs.push(ibc_msg);
                }
            }
        } else {
            let issuer_pubkey_info: JwkInfo = from_json(&isd.data_or_location)?;
            let pubkey: Jwk = from_json(&issuer_pubkey_info.jwk)?;

            if let AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                curve: EllipticCurve::Ed25519,
                ..
            }) = pubkey.algorithm
            {
                issuer_pubkeys.insert(issuer_pubkey_info.issuer, pubkey);
            } else {
                return Err(SdjwtVerifierError::UnsupportedKeyType);
            }
        }
    }

    // Update vr with the latest issuer_pubkeys
    vr.issuer_pubkeys = issuer_pubkeys.into();

    if !ibc_submsgs.is_empty() {
        Ok(_RegistrationRequest::new(vr, Some(ibc_submsgs)))
    } else {
        Ok(_RegistrationRequest::new(vr, None))
    }
}

// Functions in the `impl` block has access to the state of the contract
pub fn ibc_channel_connect_handler(
    deps: DepsMut,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, SdjwtVerifierError> {
    if CHANNEL_ID.may_load(deps.storage)?.is_some() {
        Err(SdjwtVerifierError::ChannelAlreadyExists)
    } else {
        CHANNEL_ID.save(deps.storage, &msg.channel().endpoint.channel_id)?;

        Ok(IbcBasicResponse::new())
    }
}

pub fn ibc_packet_ack_handler(
    deps: DepsMut,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, SdjwtVerifierError> {
    let (resource_req_packet, resource) = ibc_packet_ack_resource_extractor(msg)?;

    // Checks that this was a packet that we requested
    let pending_route =
        PENDING_VERIFICATION_REQ_REQUESTS.load(deps.storage, &resource_req_packet.to_string())?;
    PENDING_VERIFICATION_REQ_REQUESTS.remove(deps.storage, &resource_req_packet.to_string());

    // Checks the return data is the expected format
    let pubkeys: HashMap<String, Jwk> = from_json(resource.linked_resource.data)
        .map_err(|e| SdjwtVerifierError::ReturnedResourceFormat(e.to_string()))?;

    let mut req = APP_ROUTES_REQUIREMENTS
        .load(
            deps.storage,
            (pending_route.app_addr.clone(), pending_route.route_id),
        )
        .map_err(|_| SdjwtVerifierError::NoRequirementsForRoute)?;

    req.issuer_pubkeys = Some(pubkeys);

    APP_ROUTES_REQUIREMENTS.save(
        deps.storage,
        (pending_route.app_addr, pending_route.route_id),
        &req,
    )?;

    Ok(IbcBasicResponse::new())
}

use cosmwasm_std::{to_json_binary, Binary};
use cw_multi_test::{App, Executor};

use avida_common::types::{
    AvidaVerifierExecuteMsg, IssuerSourceOrData, RegisterRouteRequest,
    RouteVerificationRequirements,
};
use avida_sdjwt_verifier::{
    errors::SdjwtVerifierError,
    msg::QueryMsg,
    types::{Criterion, JwkInfo, ReqAttr, VerificationRequirements, CW_EXPIRATION},
};
use serde::{Deserialize, Serialize};

use super::fixtures::default_instantiate_verifier_contract;
use crate::sdjwt::fixtures::{
    get_default_presentation_required, get_input_route_requirement,
    get_two_input_routes_requirements, issuer_jwk, make_route_verification_requirements,
    ExpirationCheck, KeyType, FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID, OWNER_ADDR,
    SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn update_adding_new_jwk() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);
    let app_admin = app.api().addr_make(FIRST_CALLER_APP_ADDR);
    let default_issuer = "issuer";
    let new_jwk_issuer = "new-jwk-issuer";

    // Get existing route pubkeys
    let existing_registered_req = app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: app_admin.to_string(),
                route_id: FIRST_ROUTE_ID,
            },
        )
        .unwrap();

    let pks = existing_registered_req.issuer_pubkeys.unwrap();
    assert!(pks.len() == 1);
    assert!(pks.contains_key("issuer"));
    let jwk = pks.get("issuer").unwrap();
    let data_or_location = serde_json::to_string(&jwk).unwrap();
    // Transform the first one back into the right format
    let existing_jwk_info = JwkInfo {
        jwk: Binary::from(data_or_location.as_bytes()),
        issuer: default_issuer.to_string(),
    };

    // Create the second pubkey
    let data_or_location = serde_json::to_string(&issuer_jwk()).unwrap();
    let new_jwk_info = JwkInfo {
        jwk: Binary::from(data_or_location.as_bytes()),
        issuer: new_jwk_issuer.to_string(),
    };

    let rvr = RouteVerificationRequirements {
        issuer_source_or_data: vec![
            IssuerSourceOrData {
                source: None,
                data_or_location: to_json_binary(&new_jwk_info).unwrap(),
            },
            IssuerSourceOrData {
                source: None,
                data_or_location: to_json_binary(&existing_jwk_info).unwrap(),
            },
        ],
        presentation_required: Some(Binary::from(
            serde_json::to_string(&existing_registered_req.presentation_required)
                .unwrap()
                .as_bytes(),
        )),
    };

    // Update the route verification requirements
    app.execute_contract(
        app_admin.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Update {
            app_addr: app_admin.to_string(),
            route_id: FIRST_ROUTE_ID,
            route_criteria: Some(rvr),
        },
        &[],
    )
    .unwrap();

    // Ensure that the route verification requirements are updated
    let updated_registered_req = app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: app_admin.to_string(),
                route_id: FIRST_ROUTE_ID,
            },
        )
        .unwrap();

    let pks = updated_registered_req.issuer_pubkeys.unwrap();
    assert!(pks.len() == 2);
    assert!(pks.contains_key("issuer"));
    assert!(pks.contains_key("new-jwk-issuer"));
}

#[test]
fn update_app_not_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get route verification requirements for a single route
    let updated_req = get_default_presentation_required(ExpirationCheck::NoExpiry);
    let updated_route_verification_req =
        make_route_verification_requirements(updated_req, KeyType::Ed25519);

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try update the route verification requirements of the not registered app
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
                route_criteria: Some(updated_route_verification_req),
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::AppIsNotRegistered.to_string()));
}

#[test]
fn update_unauthorized() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Get route verification requirements for a single route
    let updated_req = get_default_presentation_required(ExpirationCheck::NoExpiry);
    let updated_route_verification_req =
        make_route_verification_requirements(updated_req, KeyType::Ed25519);

    // Update the route verification requirements using unauthorized caller address
    let err = app
        .execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
                route_criteria: Some(updated_route_verification_req),
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::UnauthorisedCaller.to_string()));
}

#[test]
fn update_serde_json_error() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Get route verification requirements for a single route
    let req = get_default_presentation_required(ExpirationCheck::Expires);
    let mut updated_route_verification_req =
        make_route_verification_requirements(req, KeyType::RSA);

    // Try update the route verification requirements with invalid presentation request
    updated_route_verification_req.presentation_required = Some(Binary::from(b"invalid"));

    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
                route_criteria: Some(updated_route_verification_req),
            },
            &[],
        )
        .unwrap_err();

    assert!(err.to_string().contains("Serialization"));
}

#[test]
fn update_unsupported_key_type() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get route verification requirements for a single route
    let req = get_default_presentation_required(ExpirationCheck::Expires);
    let route_verification_req = make_route_verification_requirements(req, KeyType::Ed25519);

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        },
        &[],
    )
    .unwrap();

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_input_route_requirement(KeyType::RSA);

    // Try update the route verification requirements with unsupported key type
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: unsupported_key_type_route_verification_requirement.route_id,
                route_criteria: Some(
                    unsupported_key_type_route_verification_requirement.requirements,
                ),
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::UnsupportedKeyType.to_string()));
}

#[test]
fn update_adding_and_remove_extra_route() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    let owner = app.api().addr_make(OWNER_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req.clone(),
        },
        &[],
    )
    .unwrap();

    // Get route verification requirements for a single route
    let updated_req = get_default_presentation_required(ExpirationCheck::Expires);
    let updated_route_verification_req =
        make_route_verification_requirements(updated_req, KeyType::Ed25519);

    // Update the route verification requirements
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Update {
            app_addr: second_caller_app_addr.to_string(),
            route_id: SECOND_ROUTE_ID,
            route_criteria: Some(updated_route_verification_req.clone()),
        },
        &[],
    )
    .unwrap();

    // Ensure that the route verification requirements are updated
    let updated_registered_req = app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    let exp = updated_registered_req
        .presentation_required
        .iter()
        .find(|attr| {
            **attr
                == ReqAttr {
                    attribute: CW_EXPIRATION.to_string(),
                    criterion: Criterion::Expires(true),
                }
        });

    assert!(exp.is_some());

    // Remove route requirements
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Update {
            app_addr: second_caller_app_addr.to_string(),
            route_id: SECOND_ROUTE_ID,
            route_criteria: None,
        },
        &[],
    )
    .unwrap();

    // Verify route requirements are removed
    assert!(app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr,
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .is_err());
}

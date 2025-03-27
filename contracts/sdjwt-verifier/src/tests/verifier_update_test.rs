use cosmwasm_std::Binary;
use cw_multi_test::{App, Executor};

use crate::errors::SdjwtVerifierError;
use avida_common::types::RegisterRouteRequest;
use serde::{Deserialize, Serialize};

use super::fixtures::default_instantiate_verifier_contract;
use crate::msg::QueryMsg;
use avida_common::types::AvidaVerifierExecuteMsg;
use avida_test_utils::sdjwt::fixtures::{
    get_default_presentation_required, get_input_route_requirement,
    get_two_input_routes_requirements, make_route_verification_requirements, ExpirationCheck,
    KeyType, OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
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
            requests: two_routes_verification_req,
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
        .query_wasm_smart::<crate::types::VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    let pks = updated_registered_req.issuer_pubkeys.unwrap();

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
        .query_wasm_smart::<crate::types::VerificationRequirements>(
            contract_addr,
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .is_err());
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

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::AppIsNotRegistered
    ));
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

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::Unauthorised
    ));
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

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::Std(_)
    ));
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

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::UnsupportedKeyType
    ));
}

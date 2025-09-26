use cosmwasm_std::Binary;
use cw_multi_test::{App, Executor};

use avida_sdjwt_verifier::{
    errors::SdjwtVerifierError, msg::QueryMsg, types::VerificationRequirements,
};

use serde::{Deserialize, Serialize};

use josekit::{self};

use super::fixtures::default_instantiate_verifier_contract;
use crate::sdjwt::fixtures::{
    claims, get_input_route_requirement, get_two_input_routes_requirements, issuer_jwk,
    make_presentation, KeyType, PresentationVerificationType, FIRST_CALLER_APP_ADDR, OWNER_ADDR,
    SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID, THIRD_ROUTE_ID,
};
use avida_common::types::AvidaVerifierExecuteMsg;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn verify_route_not_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021, None);
    let presentation = make_presentation(claims, PresentationVerificationType::Success);

    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    // Try verify presentation with not registered route
    let err = app
        .execute_contract(
            first_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(first_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::RouteNotRegistered.to_string()));
}

#[test]
fn register_success() {
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
            requests: two_routes_verification_req.clone(),
        },
        &[],
    )
    .unwrap();

    // Ensure that app is registered with the expected routes and requirements
    let registered_routes: Vec<u64> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRoutes {
                app_addr: second_caller_app_addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(registered_routes.len(), 2);

    // Query the route requirements
    let second_registered_req = app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    // Just verify that the requirements were loaded successfully
    assert!(second_registered_req.issuer_pubkeys.is_some());

    let route_verification_keys: Option<Vec<String>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteVerificationKey {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_keys.unwrap()[0]).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());

    let third_registered_req = app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: THIRD_ROUTE_ID,
            },
        )
        .unwrap();

    // Just verify that the requirements were loaded successfully
    assert!(third_registered_req.issuer_pubkeys.is_some());

    let route_verification_keys: Option<Vec<String>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetRouteVerificationKey {
                app_addr: second_caller_app_addr.to_string(),
                route_id: THIRD_ROUTE_ID,
            },
        )
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_keys.unwrap()[0]).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());
}

#[test]
fn register_app_is_already_registered() {
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
            requests: two_routes_verification_req.clone(),
        },
        &[],
    )
    .unwrap();

    // Try register the app with the two routes again
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Register {
                app_addr: second_caller_app_addr.to_string(),
                requests: two_routes_verification_req,
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::AppAlreadyRegistered.to_string()));
}

#[test]
fn register_serde_json_error() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get input verification requirements for 2 routes
    let mut two_routes_verification_req = get_two_input_routes_requirements();

    // Make invalid presentation request
    two_routes_verification_req[0]
        .requirements
        .presentation_required = Some(Binary::from(b"invalid"));

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try register the app with invalid presentation request
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Register {
                app_addr: second_caller_app_addr.to_string(),
                requests: two_routes_verification_req,
            },
            &[],
        )
        .unwrap_err();

    assert!(err.to_string().contains("Serialization"));
}

#[test]
fn register_unsupported_key_type() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_input_route_requirement(KeyType::RSA);

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try register the app with the unsupported key type
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Register {
                app_addr: second_caller_app_addr.to_string(),
                requests: vec![unsupported_key_type_route_verification_requirement],
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::UnsupportedKeyType.to_string()));
}

#[test]
fn deregister_success() {
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

    // Deregister the app
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Deregister {
            app_addr: second_caller_app_addr.to_string(),
        },
        &[],
    )
    .unwrap();

    // Ensure there is no routes left after the app deregistration
    let routes = app
        .wrap()
        .query_wasm_smart::<Vec<u64>>(
            contract_addr,
            &QueryMsg::GetRoutes {
                app_addr: second_caller_app_addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(routes, Vec::<u64>::new());
}

#[test]
fn deregister_app_not_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try deregister the not registered app
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Deregister {
                app_addr: second_caller_app_addr.to_string(),
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::AppIsNotRegistered.to_string()));
}

#[test]
fn deregister_unauthorized() {
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

    // Try deregister the app using unauthorized caller address
    let err = app
        .execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Deregister {
                app_addr: second_caller_app_addr.to_string(),
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&SdjwtVerifierError::Unauthorised.to_string()));
}

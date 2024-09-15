use cosmwasm_std::Binary;

use sylvia::multitest::App;

use crate::contract::sv::mt::SdjwtVerifierProxy;
use crate::errors::SdjwtVerifierError;
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;

use josekit::{self};

use super::fixtures::instantiate_verifier_contract;
use avida_test_utils::sdjwt::fixtures::{
    get_route_requirement_with_unsupported_key_type, get_two_input_routes_requirements, issuer_jwk,
    OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID, THIRD_ROUTE_ID,
};

#[test]
fn register_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req.clone()
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Ensure that app is registered with the expected routes and requirements
    let registered_routes = contract
        .get_routes(SECOND_CALLER_APP_ADDR.to_string())
        .unwrap();

    assert_eq!(registered_routes.len(), 2);

    let second_registered_req = contract
        .get_route_requirements(SECOND_CALLER_APP_ADDR.to_string(), SECOND_ROUTE_ID)
        .unwrap();

    assert_eq!(
        second_registered_req.issuer_source_or_data,
        two_routes_verification_req[0]
            .requirements
            .issuer_source_or_data
    );

    assert_eq!(
        second_registered_req.presentation_required,
        two_routes_verification_req[0]
            .requirements
            .presentation_required
    );

    let route_verification_key = contract
        .get_route_verification_key(SECOND_CALLER_APP_ADDR.to_string(), SECOND_ROUTE_ID)
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());

    let third_registered_req = contract
        .get_route_requirements(SECOND_CALLER_APP_ADDR.to_string(), THIRD_ROUTE_ID)
        .unwrap();

    assert_eq!(
        third_registered_req.issuer_source_or_data,
        two_routes_verification_req[1]
            .requirements
            .issuer_source_or_data
    );

    assert_eq!(
        third_registered_req.presentation_required,
        two_routes_verification_req[1]
            .requirements
            .presentation_required
    );

    let route_verification_key = contract
        .get_route_verification_key(SECOND_CALLER_APP_ADDR.to_string(), THIRD_ROUTE_ID)
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());
}

#[test]
fn register_app_is_already_registered() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req.clone()
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Try register the app with the two routes again
    assert!(matches!(
        contract
            .register(
                SECOND_CALLER_APP_ADDR.to_string(),
                two_routes_verification_req
            )
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::AppAlreadyRegistered)
    ),);
}

#[test]
fn register_serde_json_error() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let mut two_routes_verification_req = get_two_input_routes_requirements();

    // Make invalid presentation request
    two_routes_verification_req[0]
        .requirements
        .presentation_required = Binary::from(b"invalid");

    // Try register the app with the two routes and invalid presentation request
    assert!(matches!(
        contract
            .register(
                SECOND_CALLER_APP_ADDR.to_string(),
                two_routes_verification_req
            )
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::Std(_))
    ),);
}

#[test]
fn register_unsupported_key_type() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_route_requirement_with_unsupported_key_type();

    // Try egister the app with the unsupported key type
    assert!(matches!(
        contract
            .register(
                SECOND_CALLER_APP_ADDR.to_string(),
                vec![unsupported_key_type_route_verification_requirement]
            )
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::UnsupportedKeyType)
    ),);
}

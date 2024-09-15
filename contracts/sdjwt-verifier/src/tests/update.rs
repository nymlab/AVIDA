use cosmwasm_std::Binary;

use sylvia::multitest::App;

use crate::errors::SdjwtVerifierError;
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use avida_common::types::RegisterRouteRequest;

use super::fixtures::instantiate_verifier_contract;
use avida_test_utils::sdjwt::fixtures::{
    get_route_requirement_with_unsupported_key_type, get_route_verification_requirement,
    get_two_input_routes_requirements, ExpirationCheck, OWNER_ADDR, SECOND_CALLER_APP_ADDR,
    SECOND_ROUTE_ID,
};

#[test]
fn update_success() {
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

    // Get route verification requirements for a single route
    let updated_route_verification_req =
        get_route_verification_requirement(ExpirationCheck::Expires);

    // Update the route verification requirements
    assert!(contract
        .update(
            SECOND_CALLER_APP_ADDR.to_string(),
            SECOND_ROUTE_ID,
            Some(updated_route_verification_req.clone())
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Ensure that the route verification requirements are updated
    let updated_registered_req = contract
        .get_route_requirements(SECOND_CALLER_APP_ADDR.to_string(), SECOND_ROUTE_ID)
        .unwrap();

    assert_eq!(
        updated_registered_req.issuer_source_or_data,
        updated_route_verification_req.issuer_source_or_data
    );

    assert_eq!(
        updated_registered_req.presentation_required,
        updated_route_verification_req.presentation_required
    );

    // Ensure that the route verification requirements are updated
    assert!(contract
        .update(SECOND_CALLER_APP_ADDR.to_string(), SECOND_ROUTE_ID, None)
        .call(OWNER_ADDR)
        .is_ok());

    assert!(contract
        .get_route_requirements(SECOND_CALLER_APP_ADDR.to_string(), SECOND_ROUTE_ID)
        .is_err());
}

#[test]
fn update_app_not_registered() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get route verification requirements for a single route
    let updated_route_verification_req =
        get_route_verification_requirement(ExpirationCheck::NoExpiry);

    // Try update the route verification requirements of the not registered app
    assert!(matches!(
        contract
            .update(
                SECOND_CALLER_APP_ADDR.to_string(),
                SECOND_ROUTE_ID,
                Some(updated_route_verification_req)
            )
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::AppIsNotRegistered)
    ),);
}

#[test]
fn update_unathorized() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Get route verification requirements for a single route
    let updated_route_verification_req =
        get_route_verification_requirement(ExpirationCheck::NoExpiry);

    // Update the route verification requirements using unathorized caller address
    assert!(matches!(
        contract
            .update(
                SECOND_CALLER_APP_ADDR.to_string(),
                SECOND_ROUTE_ID,
                Some(updated_route_verification_req)
            )
            .call(SECOND_CALLER_APP_ADDR),
        Err(SdjwtVerifierError::Unauthorised)
    ),);
}

#[test]
fn update_serde_json_error() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Get route verification requirements for a single route
    let mut updated_route_verification_req =
        get_route_verification_requirement(ExpirationCheck::NoExpiry);

    // Try update the route verification requirements with invalid presentation request
    updated_route_verification_req.presentation_required = Binary::from(b"invalid");

    assert!(matches!(
        contract
            .update(
                SECOND_CALLER_APP_ADDR.to_string(),
                SECOND_ROUTE_ID,
                Some(updated_route_verification_req)
            )
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::Std(_))
    ),);
}

#[test]
fn update_unsupported_key_type() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get route verification requirements for a single route
    let route_verification_req = get_route_verification_requirement(ExpirationCheck::NoExpiry);

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req
            }]
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_route_requirement_with_unsupported_key_type();

    // Try update the route verification requirements with unsupported key type
    assert!(matches!(
        contract
            .update(
                SECOND_CALLER_APP_ADDR.to_string(),
                unsupported_key_type_route_verification_requirement.route_id,
                Some(unsupported_key_type_route_verification_requirement.requirements)
            )
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::UnsupportedKeyType)
    ),);
}

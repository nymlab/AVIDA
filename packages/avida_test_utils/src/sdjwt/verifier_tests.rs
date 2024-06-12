use cosmwasm_std::Binary;

use sylvia::multitest::App;

use avida_common::{
    traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy, types::InputRoutesRequirements,
};
use avida_sdjwt_verifier::{contract::sv::mt::SdjwtVerifierProxy, errors::SdjwtVerifierError};
use serde::{Deserialize, Serialize};

use josekit::{self};

use crate::sdjwt::fixtures::{
    FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID, OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID,
    THIRD_ROUTE_ID,
};

use super::fixtures::{
    claims, get_input_route_requirement, get_route_verification_requirement,
    get_two_input_routes_requirements, instantiate_verifier_contract, issuer_jwk,
    make_presentation, RouteVerificationRequirementsType, MAX_PRESENTATION_LEN,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn instantiate_success() {
    let app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, fx_route_verification_req) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    let registered_routes = contract
        .get_routes(FIRST_CALLER_APP_ADDR.to_string())
        .unwrap();

    // Ensure that app is registered with the expected routes and requirements
    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &FIRST_ROUTE_ID);

    let registered_req = contract
        .get_route_requirements(FIRST_CALLER_APP_ADDR.to_string(), FIRST_ROUTE_ID)
        .unwrap();

    assert_eq!(
        registered_req.verification_source,
        fx_route_verification_req.verification_source
    );

    assert_eq!(
        registered_req.presentation_request,
        fx_route_verification_req.presentation_request
    );

    let route_verification_key = contract
        .get_route_verification_key(FIRST_CALLER_APP_ADDR.to_string(), FIRST_ROUTE_ID)
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());
}

#[test]
fn verify_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021);

    let presentation = make_presentation(claims);

    let resp = contract
        .verify(
            Binary::from(presentation.as_bytes()),
            FIRST_ROUTE_ID,
            Some(FIRST_CALLER_APP_ADDR.to_string()),
        )
        .call(FIRST_CALLER_APP_ADDR)
        .unwrap();

    println!("resp: {:?}", resp);
}

#[test]
fn verify_presentation_too_large() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Make a presentation with a too large claims
    let claims = claims(
        &"Very long name".repeat(MAX_PRESENTATION_LEN),
        30,
        true,
        2021,
    );

    let presentation = make_presentation(claims);

    // Try verify too large presentation
    assert!(matches!(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
            )
            .call(FIRST_CALLER_APP_ADDR),
        Err(SdjwtVerifierError::PresentationTooLarge)
    ),);
}

#[test]
fn verify_route_not_registered() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021);

    let presentation = make_presentation(claims);

    // Try verify verify presentation with not registered route
    assert!(matches!(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
            )
            .call(FIRST_CALLER_APP_ADDR),
        Err(SdjwtVerifierError::RouteNotRegistered)
    ),);
}

#[test]
fn register_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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
        second_registered_req.verification_source,
        two_routes_verification_req[0]
            .requirements
            .verification_source
    );

    assert_eq!(
        second_registered_req.presentation_request,
        two_routes_verification_req[0]
            .requirements
            .presentation_request
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
        third_registered_req.verification_source,
        two_routes_verification_req[1]
            .requirements
            .verification_source
    );

    assert_eq!(
        third_registered_req.presentation_request,
        two_routes_verification_req[1]
            .requirements
            .presentation_request
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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let mut two_routes_verification_req = get_two_input_routes_requirements();

    // Make invalid presentation request
    two_routes_verification_req[0]
        .requirements
        .presentation_request = Binary::from(b"invalid");

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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_input_route_requirement(RouteVerificationRequirementsType::UnsupportedKeyType);

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

#[test]
fn deregister_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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

    // Unregister the app
    assert!(contract
        .deregister(SECOND_CALLER_APP_ADDR.to_string())
        .call(OWNER_ADDR)
        .is_ok());

    // Ensure there is no routes left after the app deregistration
    let registered_routes = contract.get_routes(SECOND_CALLER_APP_ADDR.to_string());

    assert!(registered_routes.is_err());
}

#[test]
fn deregister_app_not_registered() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Try deregister the not registered app
    assert!(matches!(
        contract
            .deregister(SECOND_CALLER_APP_ADDR.to_string(),)
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::AppIsNotRegistered)
    ),);
}

#[test]
fn deregister_unathorized() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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

    // Try deregister the app using unathorized caller address
    assert!(matches!(
        contract
            .deregister(SECOND_CALLER_APP_ADDR.to_string(),)
            .call(SECOND_CALLER_APP_ADDR),
        Err(SdjwtVerifierError::Unauthorised)
    ),);
}

#[test]
fn update_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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
        get_route_verification_requirement(RouteVerificationRequirementsType::Supported);

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
        updated_registered_req.verification_source,
        updated_route_verification_req.verification_source
    );

    assert_eq!(
        updated_registered_req.presentation_request,
        updated_route_verification_req.presentation_request
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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route
    let updated_route_verification_req =
        get_route_verification_requirement(RouteVerificationRequirementsType::Supported);

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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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
        get_route_verification_requirement(RouteVerificationRequirementsType::Supported);

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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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
        get_route_verification_requirement(RouteVerificationRequirementsType::Supported);

    // Try update the route verification requirements with invalid presentation request
    updated_route_verification_req.presentation_request = Binary::from(b"invalid");

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
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route
    let route_verification_req =
        get_route_verification_requirement(RouteVerificationRequirementsType::Supported);

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            vec![InputRoutesRequirements {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req
            }]
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_input_route_requirement(RouteVerificationRequirementsType::UnsupportedKeyType);

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

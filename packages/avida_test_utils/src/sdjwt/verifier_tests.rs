use cosmwasm_std::Binary;

use sylvia::multitest::App;

use avida_common::{
    traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy,
    types::{InputRoutesRequirements, RouteVerificationRequirements, VerificationSource},
};
use avida_sdjwt_verifier::{
    contract::{
        self,
        sv::mt::{CodeId, SdjwtVerifierProxy},
    },
    errors::SdjwtVerifierError,
    types::{Criterion, InitRegistration, MathsOperator, PresentationReq},
};
use serde::{Deserialize, Serialize};

use josekit::{self, Value};

use sd_jwt_rs::issuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTSerializationFormat};

use crate::sdjwt::fixtures::{
    FIRST_CALLER_APP_ADDR, FX_ROUTE_ID, OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID,
    THIRD_ROUTE_ID,
};

use super::fixtures::{
    claims, get_route_verification_requirement, get_two_input_routes_requirements,
    get_unsupported_key_type_input_routes_requirement, instantiate_verifier_contract, issuer,
    issuer_jwk,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn basic() {
    let app = App::default();

    let (contract, fx_route_verification_req) = instantiate_verifier_contract(&app);
    let mut fx_issuer = issuer();

    let registered_routes = contract
        .get_routes(FIRST_CALLER_APP_ADDR.to_string())
        .unwrap();

    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &FX_ROUTE_ID);

    let registered_req = contract
        .get_route_requirements(FIRST_CALLER_APP_ADDR.to_string(), FX_ROUTE_ID)
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
        .get_route_verification_key(FIRST_CALLER_APP_ADDR.to_string(), FX_ROUTE_ID)
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());

    let claims = claims("Alice", 30, true, 2021);
    let sdjwt = fx_issuer
        .issue_sd_jwt(
            claims.clone(),
            issuer::ClaimsForSelectiveDisclosureStrategy::AllLevels,
            None,
            false,
            SDJWTSerializationFormat::Compact,
        )
        .unwrap();

    let mut claims_to_disclosure = claims.clone();
    claims_to_disclosure["name"] = Value::Bool(false);
    claims_to_disclosure["age"] = Value::Bool(true);
    claims_to_disclosure["active"] = Value::Bool(true);
    claims_to_disclosure["joined_at"] = Value::Bool(true);
    let c = claims_to_disclosure.as_object().unwrap().clone();

    let mut holder = SDJWTHolder::new(sdjwt, SDJWTSerializationFormat::Compact).unwrap();
    let presentation = holder
        .create_presentation(c, None, None, None, None)
        .unwrap();

    let resp = contract
        .verify(
            Binary::from(presentation.as_bytes()),
            FX_ROUTE_ID,
            Some(FIRST_CALLER_APP_ADDR.to_string()),
        )
        .call(FIRST_CALLER_APP_ADDR)
        .unwrap();

    println!("resp: {:?}", resp);
}

/// Test the registration of a route with a presentation request
#[test]
fn register_success() {
    let app: App<_> = App::default();

    let (contract, _) = instantiate_verifier_contract(&app);

    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req.clone()
        )
        .call(OWNER_ADDR)
        .is_ok());

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

    let (contract, _) = instantiate_verifier_contract(&app);

    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req.clone()
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Register the app with the two routes again
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let mut two_routes_verification_req = get_two_input_routes_requirements();
    two_routes_verification_req[0]
        .requirements
        .presentation_request = Binary::from(b"invalid");

    // Register the app with the two routes and invalid presentation request
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let unsupported_key_type_route_verification_requirement =
        get_unsupported_key_type_input_routes_requirement();

    // TODO: Fix this test
    // Register the app with the two routes
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

    let (contract, _) = instantiate_verifier_contract(&app);

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

    let registered_routes = contract.get_routes(SECOND_CALLER_APP_ADDR.to_string());

    assert!(registered_routes.is_err());
}

#[test]
fn deregister_app_not_registered() {
    let app: App<_> = App::default();

    let (contract, _) = instantiate_verifier_contract(&app);

    // Deregister the not registered app
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Deregister the app using unathorized caller address
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req.clone()
        )
        .call(OWNER_ADDR)
        .is_ok());

    let updated_route_verification_req = get_route_verification_requirement();

    // Update the route verification requirements
    assert!(contract
        .update(
            SECOND_CALLER_APP_ADDR.to_string(),
            SECOND_ROUTE_ID,
            Some(updated_route_verification_req.clone())
        )
        .call(OWNER_ADDR)
        .is_ok());

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

    // Update the route verification requirements
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let updated_route_verification_req = get_route_verification_requirement();

    // Update the route verification requirements of the not registered app
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    let updated_route_verification_req = get_route_verification_requirement();

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

    let (contract, _) = instantiate_verifier_contract(&app);

    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    let mut updated_route_verification_req = get_route_verification_requirement();
    updated_route_verification_req.presentation_request = Binary::from(b"invalid");

    // Update the route verification requirements with invalid presentation request
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

    let (contract, _) = instantiate_verifier_contract(&app);

    let route_verification_req = get_route_verification_requirement();

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

    let unsupported_key_type_route_verification_requirement =
        get_unsupported_key_type_input_routes_requirement();

    // Update the route verification requirements with unsupported key type
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

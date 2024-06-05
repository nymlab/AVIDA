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
    claims, get_two_input_routes_requirements, instantiate_verifier_contract, issuer, issuer_jwk,
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
    assert_eq!(registered_routes, vec![SECOND_ROUTE_ID, THIRD_ROUTE_ID]);

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

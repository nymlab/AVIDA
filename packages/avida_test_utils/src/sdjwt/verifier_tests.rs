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

use crate::sdjwt::fixtures::{CALLER_APP_ADDR, FX_ROUTE_ID};

use super::fixtures::{claims, instantiate_verifier_contract, issuer, issuer_jwk};

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

    let registered_routes = contract.get_routes(CALLER_APP_ADDR.to_string()).unwrap();

    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &FX_ROUTE_ID);

    let registered_req = contract
        .get_route_requirements(CALLER_APP_ADDR.to_string(), FX_ROUTE_ID)
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
        .get_route_verification_key(CALLER_APP_ADDR.to_string(), FX_ROUTE_ID)
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
            Some(CALLER_APP_ADDR.to_string()),
        )
        .call(CALLER_APP_ADDR)
        .unwrap();

    println!("resp: {:?}", resp);
}

/// Test the registration of a route with a presentation request
#[test]
fn test_register_success() {
    let app: App<_> = App::default();
    let (contract, _) = instantiate_verifier_contract(&app);

    // contract.register(app_addr, route_criteria)
}

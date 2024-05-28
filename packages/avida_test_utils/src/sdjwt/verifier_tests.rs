use cosmwasm_std::{Addr, Binary, StdError, Uint128};
use cw_utils::Expiration;
use sylvia::multitest::App;

use avida_common::{
    traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy,
    types::{RouteVerificationRequirements, TrustRegistry, VerificationSource},
};
use avida_sdjwt_verifier::{
    contract::sv::mt::{CodeId, SdjwtVerifierProxy},
    errors::SdjwtVerifierError,
    types::{Criterion, MathsOperator, PresentationReq},
};
use serde::{Deserialize, Serialize};

use josekit::{self, Value};
use jsonwebtoken::{jwk::Jwk, DecodingKey, EncodingKey};
use sd_jwt_rs::issuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTIssuer, SDJWTJson, SDJWTSerializationFormat, SDJWTVerifier};

use super::fixtures::{claims, issuer, issuer_jwk};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn basic() {
    let app = App::default();

    let owner = "addr0001";
    let caller_app = "addr0002";
    let mut fx_issuer = issuer();

    let presentation_req: PresentationReq = vec![
        (
            "age".to_string(),
            Criterion::Number(30, MathsOperator::EqualTo),
        ),
        ("active".to_string(), Criterion::Boolean(true)),
        (
            "joined_at".to_string(),
            Criterion::Number(2020, MathsOperator::GreaterThan),
        ),
    ];

    let re = serde_json::to_string(&presentation_req).unwrap();

    let fx_jwk = serde_json::to_string(&issuer_jwk()).unwrap();
    //let fx_jwk = issuer_jwk();

    let route_id = 1u64;

    println!("fx_jwk: {:#?}", fx_jwk);

    // Add some default criteria as presentation request
    let route_verification_req: RouteVerificationRequirements = RouteVerificationRequirements {
        verification_source: VerificationSource {
            source: None,
            data_or_location: Binary::from(fx_jwk.as_bytes()),
        },
        presentation_request: Binary::from(re.as_bytes()),
    };

    let code_id = CodeId::store_code(&app);

    // String, // Admin
    // String, // App Addr
    // Vec<(RouteId, RouteVerificationRequirements)>,
    let max_presentation_len = 3000usize;
    let init_registrations = vec![(
        caller_app.to_string(), // app_admin
        caller_app.to_string(), // app_addr
        vec![(route_id, route_verification_req)],
    )];
    let contract = code_id
        .instantiate(max_presentation_len, init_registrations)
        .with_label("Verifier Contract")
        .call(owner)
        .unwrap();

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
            route_id,
            Some(caller_app.to_string()),
        )
        .call(caller_app)
        .unwrap();

    println!("resp: {:?}", resp);
}

use cosmwasm_std::Binary;

use sylvia::multitest::App;

use avida_common::{
    traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy,
    types::{InputRoutesRequirements, RouteVerificationRequirements, VerificationSource},
};
use avida_sdjwt_verifier::{
    contract::sv::mt::{CodeId, SdjwtVerifierProxy},
    types::{Criterion, InitRegistration, MathsOperator, PresentationReq},
};
use serde::{Deserialize, Serialize};

use josekit::{self, Value};

use sd_jwt_rs::issuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTSerializationFormat};

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
    let fx_route_id = 1u64;

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

    println!("fx_jwk: {:#?}", fx_jwk);

    // Add some default criteria as presentation request
    let fx_route_verification_req: RouteVerificationRequirements = RouteVerificationRequirements {
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
    let init_registrations = vec![InitRegistration {
        app_admin: caller_app.to_string(),
        app_addr: caller_app.to_string(),
        routes: vec![InputRoutesRequirements {
            route_id: fx_route_id,
            requirements: fx_route_verification_req.clone(),
        }],
    }];
    let contract = code_id
        .instantiate(max_presentation_len, init_registrations)
        .with_label("Verifier Contract")
        .call(owner)
        .unwrap();

    let registered_routes = contract.get_routes(caller_app.to_string()).unwrap();

    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &fx_route_id);

    let registered_req = contract
        .get_route_requirements(caller_app.to_string(), fx_route_id)
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
        .get_route_verification_key(caller_app.to_string(), fx_route_id)
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
            fx_route_id,
            Some(caller_app.to_string()),
        )
        .call(caller_app)
        .unwrap();

    println!("resp: {:?}", resp);
}

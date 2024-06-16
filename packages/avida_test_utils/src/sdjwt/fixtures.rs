use jsonwebtoken::EncodingKey;
use sd_jwt_rs::issuer;
use sd_jwt_rs::SDJWTIssuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTSerializationFormat};
use serde_json::Value;
use std::{fs, path::PathBuf};

use cosmwasm_std::Binary;

use avida_common::types::{
    InputRoutesRequirements, RouteVerificationRequirements, VerificationSource,
};
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq};
use josekit::{self};

/// Test constants
pub const OWNER_ADDR: &str = "addr0001";
pub const FIRST_CALLER_APP_ADDR: &str = "addr0002";
pub const SECOND_CALLER_APP_ADDR: &str = "addr0003";

pub const VERIFIER_CONTRACT_LABEL: &str = "Verifier Contract";

pub const FIRST_ROUTE_ID: u64 = 1;
pub const SECOND_ROUTE_ID: u64 = 2;
pub const THIRD_ROUTE_ID: u64 = 3;

pub const MAX_PRESENTATION_LEN: usize = 3000;

/// Is used to test different cases for route verification requirements
pub enum RouteVerificationRequirementsType {
    Supported,
    UnsupportedKeyType,
}

/// IS used to test different cases for presentation verification
pub enum PresentationVerificationType {
    Success,
    RequiredClaimsNotSatisfied,
}

// Keys generation
// ```sh
// # for Ed25519
// openssl genpkey -algorithm ED25519 -out private.pem
// openssl pkey -in private.pem -pubout -out public.pem
// ```

/// Is used to get an sdjwt issuer instance with some ed25519 predefined private key, read from a file
pub fn issuer() -> SDJWTIssuer {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let encodingkey = EncodingKey::from_ed_pem(&encoding_key_pem).unwrap();
    SDJWTIssuer::new(encodingkey, Some("EdDSA".to_string()))
}

/// Is used to get an jwk public key instance from some ed25519 predefined private key, read from a file
pub fn issuer_jwk() -> josekit::jwk::Jwk {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let key_pair = josekit::jwk::alg::ed::EdKeyPair::from_pem(encoding_key_pem).unwrap();
    key_pair.to_jwk_public_key()
}

/// Is used to get an jwk public key instance from some RSA predefined private key, read from a file
pub fn rsa_issuer_jwk() -> josekit::jwk::Jwk {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_rsa_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let key_pair = josekit::jwk::alg::rsa::RsaKeyPair::from_pem(encoding_key_pem).unwrap();
    key_pair.to_jwk_public_key()
}

pub fn claims(name: &str, age: u8, active: bool, joined_at: u16) -> Value {
    serde_json::json!({
        "exp": 1234567890,
        "iss": "issuer",
        "name": name,
        "age": age,
        "active": active,
        "joined_at": joined_at
    })
}

/// Make a presentation corresponding to the claims provided and the presentation verification error type
pub fn make_presentation(
    claims: Value,
    presentation_verification_type: PresentationVerificationType,
) -> String {
    // Get an sdjwt issuer instance with some ed25519 predefined private key, read from a file
    let mut fx_issuer = issuer();
    let sdjwt = fx_issuer
        .issue_sd_jwt(
            claims.clone(),
            issuer::ClaimsForSelectiveDisclosureStrategy::AllLevels,
            None,
            false,
            SDJWTSerializationFormat::Compact,
        )
        .unwrap();

    let mut claims_to_disclosure = claims;
    claims_to_disclosure["age"] = Value::Bool(true);
    claims_to_disclosure["active"] = Value::Bool(true);
    claims_to_disclosure["joined_at"] = Value::Bool(true);

    if let PresentationVerificationType::RequiredClaimsNotSatisfied = presentation_verification_type
    {
        claims_to_disclosure["age"] = Value::Bool(false);
    }

    let c = claims_to_disclosure.as_object().unwrap().clone();

    let mut holder = SDJWTHolder::new(sdjwt, SDJWTSerializationFormat::Compact).unwrap();
    holder
        .create_presentation(c, None, None, None, None)
        .unwrap()
}

/// Is used to get route verification requirements
pub fn make_route_verification_requirements(
    presentation_req: PresentationReq,
    route_verification_requirements_type: RouteVerificationRequirementsType,
) -> RouteVerificationRequirements {
    let re = serde_json::to_string(&presentation_req).unwrap();
    let data_or_location = match route_verification_requirements_type {
        RouteVerificationRequirementsType::Supported => {
            serde_json::to_string(&issuer_jwk()).unwrap()
        }
        RouteVerificationRequirementsType::UnsupportedKeyType => {
            serde_json::to_string(&rsa_issuer_jwk()).unwrap()
        }
    };

    let jsonwebtoken_jwk: jsonwebtoken::jwk::Jwk =
        serde_json_wasm::from_slice(data_or_location.as_bytes()).unwrap();

    println!(
        "jwk: {}",
        serde_json_wasm::to_string(&jsonwebtoken_jwk).unwrap()
    );

    // Add some default criteria as presentation request
    RouteVerificationRequirements {
        verification_source: VerificationSource {
            source: None,
            data_or_location: Binary::from(data_or_location.as_bytes()),
        },
        presentation_request: Binary::from(re.as_bytes()),
    }
}

/// Is used to get input verification requirements for 2 routes
pub fn get_two_input_routes_requirements() -> Vec<InputRoutesRequirements> {
    let first_presentation_req: PresentationReq = vec![
        ("name".to_string(), Criterion::String("John".to_string())),
        (
            "age".to_string(),
            Criterion::Number(24, MathsOperator::EqualTo),
        ),
        ("active".to_string(), Criterion::Boolean(true)),
    ];

    let second_presentation_req: PresentationReq = vec![
        ("name".to_string(), Criterion::String("Jane".to_string())),
        (
            "age".to_string(),
            Criterion::Number(30, MathsOperator::EqualTo),
        ),
        ("active".to_string(), Criterion::Boolean(true)),
    ];

    let strreq = serde_json_wasm::to_string(&second_presentation_req).unwrap();
    println!("strreq: {}", strreq);

    vec![
        InputRoutesRequirements {
            route_id: SECOND_ROUTE_ID,
            requirements: make_route_verification_requirements(
                first_presentation_req,
                RouteVerificationRequirementsType::Supported,
            ),
        },
        InputRoutesRequirements {
            route_id: THIRD_ROUTE_ID,
            requirements: make_route_verification_requirements(
                second_presentation_req,
                RouteVerificationRequirementsType::Supported,
            ),
        },
    ]
}

/// Is used to get route verification requirements for a single route
pub fn get_route_verification_requirement(
    route_verification_requirements_type: RouteVerificationRequirementsType,
) -> RouteVerificationRequirements {
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

    make_route_verification_requirements(presentation_req, route_verification_requirements_type)
}

/// Is used to get route verification requirements for a single route
pub fn get_input_route_requirement(
    route_verification_requirements_type: RouteVerificationRequirementsType,
) -> InputRoutesRequirements {
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
    InputRoutesRequirements {
        route_id: SECOND_ROUTE_ID,
        requirements: make_route_verification_requirements(
            presentation_req,
            route_verification_requirements_type,
        ),
    }
}

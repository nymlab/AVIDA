use cosmwasm_std::Binary;
use jsonwebtoken::EncodingKey;
use sd_jwt_rs::{issuer, SDJWTHolder, SDJWTIssuer, SDJWTSerializationFormat};
use serde_json::Value;
use std::{fs, path::PathBuf};

use avida_common::types::{RouteVerificationRequirements, VerificationSource};
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq};
// Keys generation
// ```sh
// # for Ed25519
// openssl genpkey -algorithm ED25519 -out private.pem
// openssl pkey -in private.pem -pubout -out public.pem
// ```

pub fn issuer() -> SDJWTIssuer {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("src/tests/keypair/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let encodingkey = EncodingKey::from_ed_pem(&encoding_key_pem).unwrap();
    SDJWTIssuer::new(encodingkey, Some("EdDSA".to_string()))
}

pub fn issuer_jwk() -> josekit::jwk::Jwk {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("src/tests/keypair/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let key_pair = josekit::jwk::alg::ed::EdKeyPair::from_pem(encoding_key_pem).unwrap();
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

pub fn create_presentation() -> String {
    let claims = claims("Alice", 30, true, 2021);
    let sdjwt = issuer()
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
    holder
        .create_presentation(c, None, None, None, None)
        .unwrap()
}

pub fn setup_requirement() -> RouteVerificationRequirements {
    // Add only 1 criterion - age greater than 18
    let presentation_req: PresentationReq = vec![
        (
            "age".to_string(),
            Criterion::Number(18, MathsOperator::GreaterThan),
        )
    ];

    let request_serialized = serde_json::to_string(&presentation_req).unwrap();
    let fx_jwk = serde_json::to_string(&issuer_jwk()).unwrap();

    println!("fx_jwk: {:#?}", fx_jwk);

    // Add some default criteria as presentation request
    RouteVerificationRequirements {
        verification_source: VerificationSource {
            source: None,
            data_or_location: Binary::from(fx_jwk.as_bytes()),
        },
        presentation_request: Binary::from(request_serialized.as_bytes()),
    }
}


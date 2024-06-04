use jsonwebtoken::EncodingKey;
use sd_jwt_rs::SDJWTIssuer;
use serde_json::Value;
use std::{fs, path::PathBuf};


use cosmwasm_std::Binary;

use sylvia::multitest::{App, Proxy};

use avida_common::{
    traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy,
    types::{InputRoutesRequirements, RouteVerificationRequirements, VerificationSource},
};
use avida_sdjwt_verifier::{
    contract::sv::mt::{CodeId, SdjwtVerifierProxy},
    contract::*,
    types::{Criterion, InitRegistration, MathsOperator, PresentationReq},
};
use serde::{Deserialize, Serialize};

use josekit::{self};

use sd_jwt_rs::issuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTSerializationFormat};
use cw_multi_test::App as MtApp;

pub const OWNER_ADDR: &str = "addr0001";
pub const CALLER_APP_ADDR: &str = "addr0002";
pub const VERIFIER_CONTRACT_LABEL: &str = "Verifier Contract";
pub const FX_ROUTE_ID: u64 = 1u64;

// Keys generation
// ```sh
// # for Ed25519
// openssl genpkey -algorithm ED25519 -out private.pem
// openssl pkey -in private.pem -pubout -out public.pem
// ```

pub fn issuer() -> SDJWTIssuer {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let encodingkey = EncodingKey::from_ed_pem(&encoding_key_pem).unwrap();
    SDJWTIssuer::new(encodingkey, Some("EdDSA".to_string()))
}

pub fn issuer_jwk() -> josekit::jwk::Jwk {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let key_pair = josekit::jwk::alg::ed::EdKeyPair::from_pem(encoding_key_pem).unwrap();
    println!("key_pair: {:#?}", key_pair);
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

pub fn instantiate_verifier_contract<'a>(app: &'a App<MtApp>) -> Proxy<'a, MtApp, SdjwtVerifier<'a>> {
    let presentation_req: PresentationReq = vec![
        (
            "age".to_string(),
            Criterion::Number(30, MathsOperator::EqualTo),
        ),
        ("active".to_string(), Criterion::Boolean(true)),
        (            "joined_at".to_string(),
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

    let code_id = CodeId::store_code(app);

    // String, // Admin
    // String, // App Addr
    // Vec<(RouteId, RouteVerificationRequirements)>,
    let max_presentation_len = 3000usize;
    let init_registrations = vec![InitRegistration {
        app_admin: CALLER_APP_ADDR.to_string(),
        app_addr: CALLER_APP_ADDR.to_string(),
        routes: vec![InputRoutesRequirements {
            route_id: FX_ROUTE_ID,
            requirements: fx_route_verification_req.clone(),
        }],
    }];

    code_id
        .instantiate(max_presentation_len, init_registrations)
        .with_label(VERIFIER_CONTRACT_LABEL)
        .call(OWNER_ADDR)
        .unwrap()
}

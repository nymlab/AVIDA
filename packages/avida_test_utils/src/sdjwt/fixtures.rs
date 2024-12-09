use avida_sdjwt_verifier::types::NumberCriterion;
use avida_sdjwt_verifier::types::ReqAttr;
use avida_sdjwt_verifier::types::IDX;
use cosmwasm_std::BlockInfo;
use cw_utils::Expiration;
use jsonwebtoken::EncodingKey;
use sd_jwt_rs::issuer;
use sd_jwt_rs::SDJWTIssuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTSerializationFormat};
use serde_json::Value;
use std::{fs, path::PathBuf};

use cosmwasm_std::{Binary, Timestamp};

use avida_common::types::{
    IssuerSourceOrData, RegisterRouteRequest, RouteVerificationRequirements,
};
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq, CW_EXPIRATION};
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

// This is the default in multitest env.block
pub const DEFAULT_TEST_BLOCKINFO: BlockInfo = BlockInfo {
    height: 12345,
    time: Timestamp::from_nanos(1571797419879305533),
    chain_id: String::new(), // default is "cosmos-testnet-14002"}
};

// This is used to define if sdjwt presentation should be expired or not
pub enum ExpirationCheck {
    Expires,
    NoExpiry,
}

/// This is used to test different cases for route verification requirements
pub enum RouteVerificationRequirementsType {
    Supported,
    UnsupportedKeyType,
}

/// This is used to test different cases for presentation verification
pub enum PresentationVerificationType {
    Success,
    OmitAgeDisclosure,
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

pub fn claims(name: &str, age: u8, active: bool, joined_at: u16, exp: Option<Expiration>) -> Value {
    let exp = match exp {
        Some(exp) => serde_json_wasm::to_string(&exp).unwrap(),
        None => "".to_string(),
    };
    serde_json::json!({
        CW_EXPIRATION: exp,
        "iss": "issuer",
        "name": name,
        "age": age,
        "active": active,
        "joined_at": joined_at
    })
}

pub fn claims_with_revocation_idx(
    name: &str,
    age: u8,
    active: bool,
    joined_at: u16,
    exp: Option<Expiration>,
    idx: u64,
) -> Value {
    let exp = match exp {
        Some(exp) => serde_json_wasm::to_string(&exp).unwrap(),
        None => "".to_string(),
    };
    serde_json::json!({
        CW_EXPIRATION: exp,
        "iss": "issuer",
        "name": name,
        "age": age,
        "active": active,
        "joined_at": joined_at,
        IDX: idx

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

    // default all claims are disclosed
    let mut claims_to_disclosure = claims;

    if let PresentationVerificationType::OmitAgeDisclosure = presentation_verification_type {
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

    // Add some default criteria as presentation request
    RouteVerificationRequirements {
        issuer_source_or_data: IssuerSourceOrData {
            source: None,
            data_or_location: Binary::from(data_or_location.as_bytes()),
        },
        presentation_required: Binary::from(re.as_bytes()),
    }
}

pub fn get_route_requirement_with_empty_revocation_list(route_id: u64) -> RegisterRouteRequest {
    let first_presentation_req: PresentationReq = vec![ReqAttr {
        attribute: IDX.to_string(),
        criterion: Criterion::NotContainedIn(vec![]),
    }];

    RegisterRouteRequest {
        route_id,
        requirements: make_route_verification_requirements(
            first_presentation_req,
            RouteVerificationRequirementsType::Supported,
        ),
    }
}

/// Is used to get input verification requirements for 2 routes
pub fn get_two_input_routes_requirements() -> Vec<RegisterRouteRequest> {
    let first_presentation_req: PresentationReq = vec![
        ReqAttr {
            attribute: "name".to_string(),
            criterion: Criterion::String("John".to_string()),
        },
        ReqAttr {
            attribute: "age".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 24,
                operator: MathsOperator::EqualTo,
            }),
        },
        ReqAttr {
            attribute: "active".to_string(),
            criterion: Criterion::Boolean(true),
        },
    ];

    let second_presentation_req: PresentationReq = vec![
        ReqAttr {
            attribute: "name".to_string(),
            criterion: Criterion::String("Jane".to_string()),
        },
        ReqAttr {
            attribute: "age".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 30,
                operator: MathsOperator::EqualTo,
            }),
        },
        ReqAttr {
            attribute: "active".to_string(),
            criterion: Criterion::Boolean(true),
        },
    ];

    let pretty_json = serde_json::to_string(&first_presentation_req).unwrap();
    println!("reg {:?}", pretty_json);

    vec![
        RegisterRouteRequest {
            route_id: SECOND_ROUTE_ID,
            requirements: make_route_verification_requirements(
                first_presentation_req,
                RouteVerificationRequirementsType::Supported,
            ),
        },
        RegisterRouteRequest {
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
    expiration_check: ExpirationCheck,
    route_verification_requirements_type: RouteVerificationRequirementsType,
) -> RouteVerificationRequirements {
    let mut presentation_req: PresentationReq = vec![
        ReqAttr {
            attribute: "age".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 30,
                operator: MathsOperator::EqualTo,
            }),
        },
        ReqAttr {
            attribute: "active".to_string(),
            criterion: Criterion::Boolean(true),
        },
        ReqAttr {
            attribute: "joined_at".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 2020,
                operator: MathsOperator::GreaterThan,
            }),
        },
    ];
    if let ExpirationCheck::Expires = expiration_check {
        presentation_req.push(ReqAttr::new(
            CW_EXPIRATION.to_string(),
            Criterion::Expires(true),
        ))
    };

    make_route_verification_requirements(presentation_req, route_verification_requirements_type)
}

/// Is used to get route verification requirements for a single route
pub fn get_input_route_requirement(
    route_verification_requirements_type: RouteVerificationRequirementsType,
) -> RegisterRouteRequest {
    let presentation_req: PresentationReq = vec![
        ReqAttr {
            attribute: "age".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 30,
                operator: MathsOperator::EqualTo,
            }),
        },
        ReqAttr {
            attribute: "active".to_string(),
            criterion: Criterion::Boolean(true),
        },
        ReqAttr {
            attribute: "joined_at".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 2020,
                operator: MathsOperator::GreaterThan,
            }),
        },
    ];
    RegisterRouteRequest {
        route_id: SECOND_ROUTE_ID,
        requirements: make_route_verification_requirements(
            presentation_req,
            route_verification_requirements_type,
        ),
    }
}

pub fn get_default_block_info() -> BlockInfo {
    DEFAULT_TEST_BLOCKINFO
}

use cosmwasm_std::{from_json, Binary};

use sylvia::multitest::App;

use crate::contract::sv::mt::SdjwtVerifierProxy;
use crate::errors::{SdjwtVerifierError, SdjwtVerifierResultError};
use crate::types::VerifyResult;
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use avida_common::types::RegisterRouteRequest;
use serde::{Deserialize, Serialize};

use super::fixtures::instantiate_verifier_contract;
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, get_route_verification_requirement, issuer_jwk,
    make_presentation, ExpirationCheck, FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID,
    MAX_PRESENTATION_LEN, OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID,
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
    let (contract, fx_route_verification_req) = instantiate_verifier_contract(&app);

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
        registered_req.issuer_source_or_data,
        fx_route_verification_req.issuer_source_or_data
    );

    assert_eq!(
        registered_req.presentation_required,
        fx_route_verification_req.presentation_required
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
fn verify_success_no_exp_validate_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021, None);

    let presentation = make_presentation(claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert!(res.result.is_ok());
}

#[test]
fn verify_success_exp_validate_success() {
    let app: App<_> = App::default();

    let (contract, _) = instantiate_verifier_contract(&app);

    // Get route verification requirements for a single route with expiration
    let route_verification_req = get_route_verification_requirement(ExpirationCheck::Expires);

    //Register the app with exp requirements
    contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        )
        .call(OWNER_ADDR)
        .unwrap();

    // Make a presentation with some claims with block time
    let valid_timestamp_claims = claims(
        "Alice",
        30,
        true,
        2021,
        Some(cw_utils::Expiration::AtTime(
            get_default_block_info().time.plus_days(1),
        )),
    );

    let presentation = make_presentation(valid_timestamp_claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(SECOND_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(SECOND_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert!(res.result.is_ok());

    // Make a presentation with some claims with block height
    let valid_blockheigh_claims = claims(
        "Alice",
        30,
        true,
        2021,
        Some(cw_utils::Expiration::AtHeight(
            get_default_block_info().height + 1,
        )),
    );

    let presentation = make_presentation(valid_blockheigh_claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(SECOND_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(SECOND_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert!(res.result.is_ok());
}

#[test]
fn verify_failed_on_expired_claim() {
    let app: App<_> = App::default();

    let (contract, _) = instantiate_verifier_contract(&app);

    // Get route verification requirements for a single route with expiration
    let route_verification_req = get_route_verification_requirement(ExpirationCheck::Expires);

    //Register the app with exp requirements
    contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        )
        .call(OWNER_ADDR)
        .unwrap();

    // Make a presentation with some claims that has expired
    let exp = cw_utils::Expiration::AtTime(get_default_block_info().time.minus_days(1));
    let invalid_timestamp_claims = claims("Alice", 30, true, 2021, Some(exp));

    let presentation = make_presentation(invalid_timestamp_claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(SECOND_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(SECOND_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.result.unwrap_err(),
        SdjwtVerifierResultError::PresentationExpired(exp)
    );

    // Make a presentation with some claims that has expired
    let exp = cw_utils::Expiration::AtHeight(get_default_block_info().height - 10);
    let invalid_blockheight_claims = claims("Alice", 30, true, 2021, Some(exp));

    let presentation = make_presentation(invalid_blockheight_claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(SECOND_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(SECOND_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.result.unwrap_err(),
        SdjwtVerifierResultError::PresentationExpired(exp)
    );
}

#[test]
fn verify_sucess_on_no_expiration_check_for_expired_claims() {
    let app: App<_> = App::default();

    let (contract, _) = instantiate_verifier_contract(&app);

    // Get route verification requirements for a single route with expiration
    let route_verification_req = get_route_verification_requirement(ExpirationCheck::NoExpiry);

    //Register the app with exp requirements
    contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        )
        .call(OWNER_ADDR)
        .unwrap();

    // Make a presentation with some claims that has expired
    let exp = cw_utils::Expiration::AtTime(get_default_block_info().time.minus_days(1));
    let invalid_timestamp_claims = claims("Alice", 30, true, 2021, Some(exp));

    let presentation = make_presentation(invalid_timestamp_claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(SECOND_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(SECOND_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert!(res.result.is_ok());
}

#[test]
fn verify_success_validate_fails() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Make a presentation with some claims that does not match presentation requirements
    let claims = claims("Alice", 30, true, 2014, None);

    let presentation = make_presentation(claims, vec![]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res.result.unwrap_err(),
        SdjwtVerifierResultError::CriterionValueFailed("joined_at".to_string())
    );
}

#[test]
fn verify_required_claims_not_satisfied() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    let claims = claims("Alice", 30, true, 2021, None);

    let presentation = make_presentation(claims, vec!["age".to_string()]);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.result.unwrap_err(),
        SdjwtVerifierResultError::DisclosedClaimNotFound(
            "Expects claim to be: Number(30, EqualTo) for key: age".to_string()
        )
    );
}

#[test]
fn verify_without_sdjwt() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Try verify presentation without sdjwt
    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(b""),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.result.unwrap_err(),
        SdjwtVerifierResultError::SdJwt("invalid input: Invalid SD-JWT length: 1".to_string())
    );
}

#[test]
fn verify_presentation_too_large() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Make a presentation with a too large claims
    let claims = claims(
        &"Very long name".repeat(MAX_PRESENTATION_LEN),
        30,
        true,
        2021,
        None,
    );

    let presentation = make_presentation(claims, vec![]);

    // Try verify too large presentation
    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.result.unwrap_err(),
        SdjwtVerifierResultError::PresentationTooLarge
    );
}

#[test]
fn verify_route_not_registered() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021, None);

    let presentation = make_presentation(claims, vec![]);

    // Try verify verify presentation with not registered route
    assert!(matches!(
        contract
            .verify(
                Binary::from(presentation.as_bytes()),
                SECOND_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR),
        Err(SdjwtVerifierError::RouteNotRegistered)
    ),);
}

use cosmwasm_std::{from_json, Binary};
use cw_multi_test::{App, Executor};

use crate::errors::SdjwtVerifierResultError;
use crate::types::VerifyResult;
use avida_common::types::RegisterRouteRequest;
use serde::{Deserialize, Serialize};

use super::fixtures::default_instantiate_verifier_contract;
use avida_common::types::AvidaVerifierExecuteMsg;
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, get_default_presentation_required, make_presentation,
    make_route_verification_requirements, ExpirationCheck, KeyType, PresentationVerificationType,
    FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID, OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn verify_success_no_exp_validate_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021, None);

    let presentation = make_presentation(claims, PresentationVerificationType::Success);

    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    let res: VerifyResult = from_json(
        app.execute_contract(
            first_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: FIRST_ROUTE_ID,
                app_addr: Some(first_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert!(res.success);
}

#[test]
fn verify_success_exp_validate_success() {
    let mut app = App::default();

    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get route verification requirements for a single route with expiration
    let req = get_default_presentation_required(ExpirationCheck::Expires);
    let route_verification_req = make_route_verification_requirements(req, KeyType::Ed25519);

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    //Register the app with exp requirements
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        },
        &[],
    )
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

    let presentation = make_presentation(
        valid_timestamp_claims,
        PresentationVerificationType::Success,
    );

    let res: VerifyResult = from_json(
        app.execute_contract(
            second_caller_app_addr.clone(),
            contract_addr.clone(),
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(second_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert!(res.success);

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

    let presentation = make_presentation(
        valid_blockheigh_claims,
        PresentationVerificationType::Success,
    );

    let res: VerifyResult = from_json(
        app.execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(second_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert!(res.success);
}

#[test]
fn verify_failed_on_expired_claim() {
    let mut app = App::default();

    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get route verification requirements for a single route with expiration
    let req = get_default_presentation_required(ExpirationCheck::Expires);
    let route_verification_req = make_route_verification_requirements(req, KeyType::Ed25519);

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    //Register the app with exp requirements
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        },
        &[],
    )
    .unwrap();

    // Make a presentation with some claims that has expired
    let exp = cw_utils::Expiration::AtTime(get_default_block_info().time.minus_days(1));
    let invalid_timestamp_claims = claims("Alice", 30, true, 2021, Some(exp));

    let presentation = make_presentation(
        invalid_timestamp_claims,
        PresentationVerificationType::Success,
    );

    let res: VerifyResult = from_json(
        app.execute_contract(
            second_caller_app_addr.clone(),
            contract_addr.clone(),
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(second_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.error.unwrap(),
        SdjwtVerifierResultError::PresentationExpired(exp).to_string()
    );

    // Make a presentation with some claims that has expired
    let exp = cw_utils::Expiration::AtHeight(get_default_block_info().height - 10);
    let invalid_blockheight_claims = claims("Alice", 30, true, 2021, Some(exp));

    let presentation = make_presentation(
        invalid_blockheight_claims,
        PresentationVerificationType::Success,
    );

    let res: VerifyResult = from_json(
        app.execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(second_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res.error.unwrap(),
        SdjwtVerifierResultError::PresentationExpired(exp).to_string()
    );
}

#[test]
fn verify_success_on_no_expiration_check_for_expired_claims() {
    let mut app = App::default();

    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Get route verification requirements for a single route with expiration
    let req = get_default_presentation_required(ExpirationCheck::NoExpiry);
    let route_verification_req = make_route_verification_requirements(req, KeyType::Ed25519);

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    //Register the app with exp requirements
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: vec![RegisterRouteRequest {
                route_id: SECOND_ROUTE_ID,
                requirements: route_verification_req,
            }],
        },
        &[],
    )
    .unwrap();

    // Make a presentation with some claims that has expired
    let exp = cw_utils::Expiration::AtTime(get_default_block_info().time.minus_days(1));
    let invalid_timestamp_claims = claims("Alice", 30, true, 2021, Some(exp));

    let presentation = make_presentation(
        invalid_timestamp_claims,
        PresentationVerificationType::Success,
    );

    let res: VerifyResult = from_json(
        app.execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(second_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert!(res.success);
}

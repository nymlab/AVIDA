use cosmwasm_std::{from_json, Binary};
use cw_multi_test::{App, Executor};

use avida_sdjwt_verifier::errors::SdjwtVerifierResultError;
use avida_sdjwt_verifier::types::VerifyResult;
use serde::{Deserialize, Serialize};

use super::fixtures::default_instantiate_verifier_contract;
use crate::sdjwt::fixtures::{
    claims, make_presentation, PresentationVerificationType, FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID,
    MAX_PRESENTATION_LEN,
};
use avida_common::types::AvidaVerifierExecuteMsg;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn verify_success_incorrect_claims_validate_fails() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Make a presentation with some claims that does not match presentation requirements
    // Criteria (age == 30, active == true, joined_at > 2020)
    // Criterion joined_at > 2020 will fail
    let claims = claims("Alice", 30, true, 2014, None);
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

    assert_eq!(
        res.error.unwrap(),
        SdjwtVerifierResultError::CriterionValueFailed("joined_at".to_string()).to_string()
    );
}

#[test]
fn verify_required_claims_not_satisfied() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    let claims = claims("Alice", 30, true, 2021, None);
    // We omit age disclosure to fail the criteria
    // Criteria (age == 30, active == true, joined_at > 2020)
    let presentation = make_presentation(claims, PresentationVerificationType::OmitAgeDisclosure);

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

    assert_eq!(
        res.error.unwrap(),
        SdjwtVerifierResultError::DisclosedClaimNotFound(
            "Expects claim to be: Number(NumberCriterion { value: 30, operator: EqualTo }) for attr: age".to_string()
        ).to_string()
    );
}

#[test]
fn verify_without_sdjwt() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    // Try verify presentation without sdjwt
    let res: VerifyResult = from_json(
        app.execute_contract(
            first_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(b""),
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

    assert_eq!(
        res.error.unwrap(),
        SdjwtVerifierResultError::SdJwtRsError(
            "invalid input: Invalid SD-JWT length: 1".to_string()
        )
        .to_string()
    );
}

#[test]
fn verify_presentation_too_large() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

    // Make a presentation with a too large claims
    let claims = claims(
        &"Very long name".repeat(MAX_PRESENTATION_LEN),
        30,
        true,
        2021,
        None,
    );

    let presentation = make_presentation(claims, PresentationVerificationType::Success);
    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    // Try verify too large presentation
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

    assert_eq!(
        res.error.unwrap(),
        SdjwtVerifierResultError::PresentationTooLarge.to_string()
    );
}

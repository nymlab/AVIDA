use cosmwasm_std::{from_json, Binary};
use cw_multi_test::{App, Executor};

use crate::errors::{SdjwtVerifierError, SdjwtVerifierResultError};
use crate::types::VerifyResult;
use avida_common::types::{RegisterRouteRequest, RouteVerificationRequirements};
use serde::{Deserialize, Serialize};

use josekit::{self};

use super::fixtures::instantiate_verifier_contract;
use crate::msg::QueryMsg;
use avida_common::types::AvidaVerifierExecuteMsg;
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, get_input_route_requirement,
    get_route_verification_requirement, get_two_input_routes_requirements, issuer_jwk,
    make_presentation, ExpirationCheck, PresentationVerificationType,
    RouteVerificationRequirementsType, FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID, MAX_PRESENTATION_LEN,
    OWNER_ADDR, SECOND_CALLER_APP_ADDR, SECOND_ROUTE_ID, THIRD_ROUTE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn instantiate_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, fx_route_verification_req) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    let registered_routes: Vec<u64> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRoutes {
                app_addr: first_caller_app_addr.to_string(),
            },
        )
        .unwrap();

    // Ensure that app is registered with the expected routes and requirements
    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &FIRST_ROUTE_ID);

    let registered_req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: first_caller_app_addr.to_string(),
                route_id: FIRST_ROUTE_ID,
            },
        )
        .unwrap();

    assert_eq!(
        registered_req.issuer_source_or_data,
        fx_route_verification_req.issuer_source_or_data
    );

    assert_eq!(
        registered_req.presentation_required,
        fx_route_verification_req.presentation_required
    );

    let route_verification_key: Option<String> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetRouteVerificationKey {
                app_addr: first_caller_app_addr.to_string(),
                route_id: FIRST_ROUTE_ID,
            },
        )
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key.unwrap()).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());
}

#[test]
fn verify_success_no_exp_validate_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

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

    assert!(res.result.is_ok());
}

#[test]
fn verify_success_exp_validate_success() {
    let mut app = App::default();

    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req = get_route_verification_requirement(
        ExpirationCheck::Expires,
        RouteVerificationRequirementsType::Supported,
    );

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

    assert!(res.result.is_ok());
}

#[test]
fn verify_failed_on_expired_claim() {
    let mut app = App::default();

    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req = get_route_verification_requirement(
        ExpirationCheck::Expires,
        RouteVerificationRequirementsType::Supported,
    );

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
        res.result.unwrap_err(),
        SdjwtVerifierResultError::PresentationExpired(exp)
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
        res.result.unwrap_err(),
        SdjwtVerifierResultError::PresentationExpired(exp)
    );
}

#[test]
fn verify_route_not_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Make a presentation with some claims
    let claims = claims("Alice", 30, true, 2021, None);
    let presentation = make_presentation(claims, PresentationVerificationType::Success);

    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    // Try verify presentation with not registered route
    let err = app
        .execute_contract(
            first_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Verify {
                presentation: Binary::from(presentation.as_bytes()),
                route_id: SECOND_ROUTE_ID,
                app_addr: Some(first_caller_app_addr.to_string()),
                additional_requirements: None,
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::RouteNotRegistered
    ));
}

#[test]
fn verify_success_validate_fails() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Make a presentation with some claims that does not match presentation requirements
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
        res.result.unwrap_err(),
        SdjwtVerifierResultError::CriterionValueFailed("joined_at".to_string())
    );
}

#[test]
fn verify_required_claims_not_satisfied() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    let claims = claims("Alice", 30, true, 2021, None);
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
        res.result.unwrap_err(),
        SdjwtVerifierResultError::DisclosedClaimNotFound(
            "Expects claim to be: Number(NumberCriterion { value: 30, operator: EqualTo }) for attr: age".to_string()
        )
    );
}

#[test]
fn verify_without_sdjwt() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

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
        res.result.unwrap_err(),
        SdjwtVerifierResultError::SdJwt("invalid input: Invalid SD-JWT length: 1".to_string())
    );
}

#[test]
fn verify_presentation_too_large() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

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
        res.result.unwrap_err(),
        SdjwtVerifierResultError::PresentationTooLarge
    );
}

#[test]
fn verify_success_on_no_expiration_check_for_expired_claims() {
    let mut app = App::default();

    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req = get_route_verification_requirement(
        ExpirationCheck::NoExpiry,
        RouteVerificationRequirementsType::Supported,
    );

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

    assert!(res.result.is_ok());
}

#[test]
fn register_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req.clone(),
        },
        &[],
    )
    .unwrap();

    // Ensure that app is registered with the expected routes and requirements
    let registered_routes: Vec<u64> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRoutes {
                app_addr: second_caller_app_addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(registered_routes.len(), 2);

    let second_registered_req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    assert_eq!(
        second_registered_req.issuer_source_or_data,
        two_routes_verification_req[0]
            .requirements
            .issuer_source_or_data
    );

    assert_eq!(
        second_registered_req.presentation_required,
        two_routes_verification_req[0]
            .requirements
            .presentation_required
    );

    let route_verification_key: Option<String> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteVerificationKey {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key.unwrap()).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());

    let third_registered_req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: THIRD_ROUTE_ID,
            },
        )
        .unwrap();

    assert_eq!(
        third_registered_req.issuer_source_or_data,
        two_routes_verification_req[1]
            .requirements
            .issuer_source_or_data
    );

    assert_eq!(
        third_registered_req.presentation_required,
        two_routes_verification_req[1]
            .requirements
            .presentation_required
    );

    let route_verification_key: Option<String> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetRouteVerificationKey {
                app_addr: second_caller_app_addr.to_string(),
                route_id: THIRD_ROUTE_ID,
            },
        )
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key.unwrap()).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());
}

#[test]
fn register_app_is_already_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);
    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req.clone(),
        },
        &[],
    )
    .unwrap();

    // Try register the app with the two routes again
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Register {
                app_addr: second_caller_app_addr.to_string(),
                requests: two_routes_verification_req,
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::AppAlreadyRegistered
    ));
}

#[test]
fn register_serde_json_error() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let mut two_routes_verification_req = get_two_input_routes_requirements();

    // Make invalid presentation request
    two_routes_verification_req[0]
        .requirements
        .presentation_required = Some(Binary::from(b"invalid"));

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try register the app with invalid presentation request
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Register {
                app_addr: second_caller_app_addr.to_string(),
                requests: two_routes_verification_req,
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::Std(_)
    ));
}

#[test]
fn register_unsupported_key_type() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_input_route_requirement(RouteVerificationRequirementsType::UnsupportedKeyType);

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try register the app with the unsupported key type
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Register {
                app_addr: second_caller_app_addr.to_string(),
                requests: vec![unsupported_key_type_route_verification_requirement],
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::UnsupportedKeyType
    ));
}

#[test]
fn deregister_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Deregister the app
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Deregister {
            app_addr: second_caller_app_addr.to_string(),
        },
        &[],
    )
    .unwrap();

    // Ensure there is no routes left after the app deregistration
    let err = app
        .wrap()
        .query_wasm_smart::<Vec<u64>>(
            contract_addr,
            &QueryMsg::GetRoutes {
                app_addr: second_caller_app_addr.to_string(),
            },
        )
        .unwrap_err();

    assert!(err.to_string().contains("not found"));
}

#[test]
fn deregister_app_not_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    let owner = app.api().addr_make(OWNER_ADDR);

    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try deregister the not registered app
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Deregister {
                app_addr: second_caller_app_addr.to_string(),
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::AppIsNotRegistered
    ));
}

#[test]
fn deregister_unauthorized() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Try deregister the app using unauthorized caller address
    let err = app
        .execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Deregister {
                app_addr: second_caller_app_addr.to_string(),
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::Unauthorised
    ));
}

#[test]
fn update_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    let owner = app.api().addr_make(OWNER_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Get route verification requirements for a single route
    let updated_route_verification_req = get_route_verification_requirement(
        ExpirationCheck::Expires,
        RouteVerificationRequirementsType::Supported,
    );

    // Update the route verification requirements
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Update {
            app_addr: second_caller_app_addr.to_string(),
            route_id: SECOND_ROUTE_ID,
            route_criteria: Some(updated_route_verification_req.clone()),
        },
        &[],
    )
    .unwrap();

    // Ensure that the route verification requirements are updated
    let updated_registered_req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .unwrap();

    assert_eq!(
        updated_registered_req.issuer_source_or_data,
        updated_route_verification_req.issuer_source_or_data
    );

    assert_eq!(
        updated_registered_req.presentation_required,
        updated_route_verification_req.presentation_required
    );

    // Remove route requirements
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Update {
            app_addr: second_caller_app_addr.to_string(),
            route_id: SECOND_ROUTE_ID,
            route_criteria: None,
        },
        &[],
    )
    .unwrap();

    // Verify route requirements are removed
    assert!(app
        .wrap()
        .query_wasm_smart::<RouteVerificationRequirements>(
            contract_addr,
            &QueryMsg::GetRouteRequirements {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
            },
        )
        .is_err());
}

#[test]
fn update_app_not_registered() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route
    let updated_route_verification_req = get_route_verification_requirement(
        ExpirationCheck::NoExpiry,
        RouteVerificationRequirementsType::Supported,
    );

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Try update the route verification requirements of the not registered app
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
                route_criteria: Some(updated_route_verification_req),
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::AppIsNotRegistered
    ));
}

#[test]
fn update_unauthorized() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner,
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Get route verification requirements for a single route
    let updated_route_verification_req = get_route_verification_requirement(
        ExpirationCheck::NoExpiry,
        RouteVerificationRequirementsType::Supported,
    );

    // Update the route verification requirements using unauthorized caller address
    let err = app
        .execute_contract(
            second_caller_app_addr.clone(),
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
                route_criteria: Some(updated_route_verification_req),
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::Unauthorised
    ));
}

#[test]
fn update_serde_json_error() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
    app.execute_contract(
        owner.clone(),
        contract_addr.clone(),
        &AvidaVerifierExecuteMsg::Register {
            app_addr: second_caller_app_addr.to_string(),
            requests: two_routes_verification_req,
        },
        &[],
    )
    .unwrap();

    // Get route verification requirements for a single route
    let mut updated_route_verification_req = get_route_verification_requirement(
        ExpirationCheck::NoExpiry,
        RouteVerificationRequirementsType::Supported,
    );

    // Try update the route verification requirements with invalid presentation request
    updated_route_verification_req.presentation_required = Some(Binary::from(b"invalid"));

    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: SECOND_ROUTE_ID,
                route_criteria: Some(updated_route_verification_req),
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::Std(_)
    ));
}

#[test]
fn update_unsupported_key_type() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route
    let route_verification_req = get_route_verification_requirement(
        ExpirationCheck::NoExpiry,
        RouteVerificationRequirementsType::Supported,
    );

    let owner = app.api().addr_make(OWNER_ADDR);
    let second_caller_app_addr = app.api().addr_make(SECOND_CALLER_APP_ADDR);

    // Register the app with the two routes
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

    // Get an unsupported input verification requirements for a single route
    let unsupported_key_type_route_verification_requirement =
        get_input_route_requirement(RouteVerificationRequirementsType::UnsupportedKeyType);

    // Try update the route verification requirements with unsupported key type
    let err = app
        .execute_contract(
            owner,
            contract_addr,
            &AvidaVerifierExecuteMsg::Update {
                app_addr: second_caller_app_addr.to_string(),
                route_id: unsupported_key_type_route_verification_requirement.route_id,
                route_criteria: Some(
                    unsupported_key_type_route_verification_requirement.requirements,
                ),
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast().unwrap(),
        SdjwtVerifierError::UnsupportedKeyType
    ));
}

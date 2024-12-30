use avida_common::types::RouteVerificationRequirements;
use cosmwasm_std::{from_json, to_json_binary, Binary};
use cw_multi_test::{App, Executor};

use crate::errors::SdjwtVerifierResultError;
use crate::types::{Criterion, PresentationReq, ReqAttr, VerifyResult};
use serde::{Deserialize, Serialize};

use super::fixtures::instantiate_verifier_contract;
use crate::msg::{ExecuteMsg, QueryMsg};
use avida_test_utils::sdjwt::fixtures::{
    claims_with_revocation_idx, get_route_requirement_with_empty_revocation_list,
    make_presentation, PresentationVerificationType, RouteVerificationRequirementsType,
    FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub idx: u64,
}

const REVOCATION_ROUTE_ID: u64 = 100;
const REVOCATION_TEST_CALLER: &str = "revocation_test_caller";

#[test]
fn test_update_revocation_list() {
    let mut app = App::default();

    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req =
        get_route_requirement_with_empty_revocation_list(REVOCATION_ROUTE_ID);

    let revocation_test_caller = app.api().addr_make(REVOCATION_TEST_CALLER);

    // Register the app with exp requirements
    let register_app_msg = ExecuteMsg::Register {
        app_addr: revocation_test_caller.to_string(),
        requests: vec![route_verification_req.clone()],
    };

    app.execute_contract(
        revocation_test_caller.clone(),
        contract_addr.clone(),
        &register_app_msg,
        &[],
    )
    .unwrap();

    let req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: revocation_test_caller.to_string(),
                route_id: REVOCATION_ROUTE_ID,
            },
        )
        .unwrap();

    let presentation_required: PresentationReq =
        from_json(req.presentation_required.unwrap()).unwrap();

    let revocation_list = presentation_required
        .iter()
        .find(|req| req.attribute == "idx")
        .unwrap();
    assert_eq!(revocation_list.criterion, Criterion::NotContainedIn(vec![]));

    // Update revocation list
    let update_revocation_list_msg = ExecuteMsg::UpdateRevocationList {
        app_addr: revocation_test_caller.to_string(),
        request: crate::types::UpdateRevocationListRequest {
            route_id: REVOCATION_ROUTE_ID,
            revoke: vec![1, 2, 3],
            unrevoke: vec![4, 5],
        },
    };
    app.execute_contract(
        revocation_test_caller.clone(),
        contract_addr.clone(),
        &update_revocation_list_msg,
        &[],
    )
    .unwrap();

    let req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: revocation_test_caller.to_string(),
                route_id: REVOCATION_ROUTE_ID,
            },
        )
        .unwrap();

    let presentation_required: PresentationReq =
        from_json(req.presentation_required.unwrap()).unwrap();

    let revocation_list = presentation_required
        .iter()
        .find(|req| req.attribute == "idx")
        .unwrap();
    assert_eq!(
        revocation_list.criterion,
        Criterion::NotContainedIn(vec![1, 2, 3])
    );

    let update_revocation_list_msg = ExecuteMsg::UpdateRevocationList {
        app_addr: revocation_test_caller.to_string(),
        request: crate::types::UpdateRevocationListRequest {
            route_id: REVOCATION_ROUTE_ID,
            revoke: vec![7, 1, 7],
            unrevoke: vec![2, 5],
        },
    };

    app.execute_contract(
        revocation_test_caller.clone(),
        contract_addr.clone(),
        &update_revocation_list_msg,
        &[],
    )
    .unwrap();

    let req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetRouteRequirements {
                app_addr: revocation_test_caller.to_string(),
                route_id: REVOCATION_ROUTE_ID,
            },
        )
        .unwrap();

    let presentation_required: PresentationReq =
        from_json(req.presentation_required.unwrap()).unwrap();

    let revocation_list = presentation_required
        .iter()
        .find(|req| req.attribute == "idx")
        .unwrap();
    assert_eq!(
        revocation_list.criterion,
        Criterion::NotContainedIn(vec![1, 3, 7])
    );
}

#[test]
fn test_revoked_presentation_cannot_be_used() {
    let revoked_idx = 111;
    let unrevoked_idx = 222;

    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req =
        get_route_requirement_with_empty_revocation_list(REVOCATION_ROUTE_ID);

    let revocation_test_caller = app.api().addr_make(REVOCATION_TEST_CALLER);

    // Register the app with exp requirements
    let register_app_msg = ExecuteMsg::Register {
        app_addr: revocation_test_caller.to_string(),
        requests: vec![route_verification_req.clone()],
    };

    app.execute_contract(
        revocation_test_caller.clone(),
        contract_addr.clone(),
        &register_app_msg,
        &[],
    )
    .unwrap();

    let update_revocation_list_msg = ExecuteMsg::UpdateRevocationList {
        app_addr: revocation_test_caller.to_string(),
        request: crate::types::UpdateRevocationListRequest {
            route_id: REVOCATION_ROUTE_ID,
            revoke: vec![revoked_idx],
            unrevoke: vec![unrevoked_idx],
        },
    };

    app.execute_contract(
        revocation_test_caller.clone(),
        contract_addr.clone(),
        &update_revocation_list_msg,
        &[],
    )
    .unwrap();

    // Make a presentation with some claims
    let revoked_claims = claims_with_revocation_idx("Alice", 30, true, 2021, None, revoked_idx);

    let unrevoked_claims = claims_with_revocation_idx("Alice", 30, true, 2021, None, unrevoked_idx);

    let revoked_presentation =
        make_presentation(revoked_claims, PresentationVerificationType::Success);
    let valid_presentation =
        make_presentation(unrevoked_claims, PresentationVerificationType::Success);

    let verify_msg = ExecuteMsg::Verify {
        presentation: Binary::from(revoked_presentation.as_bytes()),
        route_id: REVOCATION_ROUTE_ID,
        app_addr: Some(revocation_test_caller.to_string()),
        additional_requirements: None,
    };

    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    let res = app
        .execute_contract(
            first_caller_app_addr.clone(),
            contract_addr.clone(),
            &verify_msg,
            &[],
        )
        .unwrap();
    let verify_res: VerifyResult = from_json(res.data.unwrap()).unwrap();
    let err = verify_res.result.unwrap_err();

    assert_eq!(err, SdjwtVerifierResultError::IdxRevoked(revoked_idx));

    let verify_msg = ExecuteMsg::Verify {
        presentation: Binary::from(valid_presentation.as_bytes()),
        route_id: REVOCATION_ROUTE_ID,
        app_addr: Some(revocation_test_caller.to_string()),
        additional_requirements: None,
    };
    let res: VerifyResult = from_json(
        app.execute_contract(first_caller_app_addr, contract_addr, &verify_msg, &[])
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();

    assert!(res.result.is_ok());
}

#[test]
fn test_addition_requirements_with_revocation_list() {
    let revoked_idx = 111;

    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    // By default there is no revocation list
    let (contract_addr, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Now we create additional requirements for the route
    let addition_requirement = vec![ReqAttr {
        attribute: "idx".to_string(),
        criterion: Criterion::NotContainedIn(vec![revoked_idx]),
    }];

    // Make a presentation with some claims
    let revoked_claims = claims_with_revocation_idx("Alice", 30, true, 2021, None, revoked_idx);

    let revoked_presentation =
        make_presentation(revoked_claims, PresentationVerificationType::Success);

    // Additional requirements should be checked if revoked_claims is revoked and should error
    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);

    let verify_msg = ExecuteMsg::Verify {
        presentation: Binary::from(revoked_presentation.as_bytes()),
        route_id: FIRST_ROUTE_ID,
        app_addr: Some(first_caller_app_addr.to_string()),
        additional_requirements: Some(to_json_binary(&addition_requirement).unwrap()),
    };
    let res: VerifyResult = from_json(
        app.execute_contract(
            first_caller_app_addr.clone(),
            contract_addr.clone(),
            &verify_msg,
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();
    let err = res.result.unwrap_err();
    assert_eq!(err, SdjwtVerifierResultError::IdxRevoked(revoked_idx));

    // Additional requirements not present, revoked_claims is not checked and should ok
    let verify_msg = ExecuteMsg::Verify {
        presentation: Binary::from(revoked_presentation.as_bytes()),
        route_id: FIRST_ROUTE_ID,
        app_addr: Some(first_caller_app_addr.to_string()),
        additional_requirements: None,
    };
    let res: VerifyResult = from_json(
        app.execute_contract(
            first_caller_app_addr,
            contract_addr.clone(),
            &verify_msg,
            &[],
        )
        .unwrap()
        .data
        .unwrap(),
    )
    .unwrap();

    assert!(res.result.is_ok());
}

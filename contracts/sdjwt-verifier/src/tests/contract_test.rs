use cosmwasm_std::{from_json, to_json_binary, Binary};

use sylvia::multitest::App;

use crate::contract::sv::mt::SdjwtVerifierProxy;
use crate::errors::SdjwtVerifierResultError;
use crate::types::{Criterion, PresentationReq, ReqAttr, VerifyResult};
use serde::{Deserialize, Serialize};

use super::fixtures::instantiate_verifier_contract;
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
    let app: App<_> = App::default();

    let (contract, _) =
        instantiate_verifier_contract(&mut app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req =
        get_route_requirement_with_empty_revocation_list(REVOCATION_ROUTE_ID);

    //Register the app with exp requirements
    contract
        .register(
            REVOCATION_TEST_CALLER.to_string(),
            vec![route_verification_req.clone()],
        )
        .call(REVOCATION_TEST_CALLER)
        .unwrap();

    let req: PresentationReq = from_json(
        contract
            .get_route_requirements(REVOCATION_TEST_CALLER.to_string(), REVOCATION_ROUTE_ID)
            .unwrap()
            .presentation_required
            .unwrap(),
    )
    .unwrap();

    let revocation_list = req.iter().find(|req| req.attribute == "idx").unwrap();
    assert_eq!(revocation_list.criterion, Criterion::NotContainedIn(vec![]));

    contract
        .update_revocation_list(
            REVOCATION_TEST_CALLER.to_string(),
            crate::types::UpdateRevocationListRequest {
                route_id: REVOCATION_ROUTE_ID,
                revoke: vec![1, 2, 3],
                unrevoke: vec![4, 5],
            },
        )
        .call(REVOCATION_TEST_CALLER)
        .unwrap();

    let req: PresentationReq = from_json(
        contract
            .get_route_requirements(REVOCATION_TEST_CALLER.to_string(), REVOCATION_ROUTE_ID)
            .unwrap()
            .presentation_required
            .unwrap(),
    )
    .unwrap();

    let revocation_list = req.iter().find(|req| req.attribute == "idx").unwrap();
    assert_eq!(
        revocation_list.criterion,
        Criterion::NotContainedIn(vec![1, 2, 3])
    );

    contract
        .update_revocation_list(
            REVOCATION_TEST_CALLER.to_string(),
            crate::types::UpdateRevocationListRequest {
                route_id: REVOCATION_ROUTE_ID,
                revoke: vec![7, 1, 7],
                unrevoke: vec![2, 5],
            },
        )
        .call(REVOCATION_TEST_CALLER)
        .unwrap();

    let req: PresentationReq = from_json(
        contract
            .get_route_requirements(REVOCATION_TEST_CALLER.to_string(), REVOCATION_ROUTE_ID)
            .unwrap()
            .presentation_required
            .unwrap(),
    )
    .unwrap();

    let revocation_list = req.iter().find(|req| req.attribute == "idx").unwrap();
    assert_eq!(
        revocation_list.criterion,
        Criterion::NotContainedIn(vec![1, 3, 7])
    );
}

#[test]
fn test_revoked_presentation_cannot_be_used() {
    let revoked_idx = 111;
    let unrevoked_idx = 222;

    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

    // Get route verification requirements for a single route with expiration
    let route_verification_req =
        get_route_requirement_with_empty_revocation_list(REVOCATION_ROUTE_ID);

    //Register the app with exp requirements
    contract
        .register(
            REVOCATION_TEST_CALLER.to_string(),
            vec![route_verification_req.clone()],
        )
        .call(REVOCATION_TEST_CALLER)
        .unwrap();

    contract
        .update_revocation_list(
            REVOCATION_TEST_CALLER.to_string(),
            crate::types::UpdateRevocationListRequest {
                route_id: REVOCATION_ROUTE_ID,
                revoke: vec![revoked_idx],
                unrevoke: vec![unrevoked_idx],
            },
        )
        .call(REVOCATION_TEST_CALLER)
        .unwrap();

    // Make a presentation with some claims
    let revoked_claims = claims_with_revocation_idx("Alice", 30, true, 2021, None, revoked_idx);

    let unrevoked_claims = claims_with_revocation_idx("Alice", 30, true, 2021, None, unrevoked_idx);

    let revoked_presentation =
        make_presentation(revoked_claims, PresentationVerificationType::Success);
    let valid_presentation =
        make_presentation(unrevoked_claims, PresentationVerificationType::Success);

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(revoked_presentation.as_bytes()),
                REVOCATION_ROUTE_ID,
                Some(REVOCATION_TEST_CALLER.to_string()),
                None,
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();
    let err = res.result.unwrap_err();

    assert_eq!(err, SdjwtVerifierResultError::IdxRevoked(revoked_idx));

    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(valid_presentation.as_bytes()),
                REVOCATION_ROUTE_ID,
                Some(REVOCATION_TEST_CALLER.to_string()),
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
fn test_addition_requirements_with_revocation_list() {
    let revoked_idx = 111;

    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    // By default there is no revocation list
    let (contract, _) =
        instantiate_verifier_contract(&app, RouteVerificationRequirementsType::Supported);

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
    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(revoked_presentation.as_bytes()),
                FIRST_ROUTE_ID,
                Some(FIRST_CALLER_APP_ADDR.to_string()),
                Some(to_json_binary(&addition_requirement).unwrap()),
            )
            .call(FIRST_CALLER_APP_ADDR)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();
    let err = res.result.unwrap_err();
    assert_eq!(err, SdjwtVerifierResultError::IdxRevoked(revoked_idx));

    // Additional requirements not present, revoked_claims is not checked and should ok
    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(revoked_presentation.as_bytes()),
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

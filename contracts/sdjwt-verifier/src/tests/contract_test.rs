use cosmwasm_std::{from_json, Binary};

use sylvia::multitest::App;

use crate::contract::sv::mt::SdjwtVerifierProxy;
use crate::types::{Criterion, PresentationReq};
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use serde::{Deserialize, Serialize};

use super::fixtures::instantiate_verifier_contract;
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, get_input_route_requirement,
    get_route_requirement_with_empty_revocation_list, get_route_verification_requirement,
    get_two_input_routes_requirements, issuer_jwk, make_presentation, ExpirationCheck,
    PresentationVerificationType, RouteVerificationRequirementsType, FIRST_CALLER_APP_ADDR,
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

    let req: PresentationReq = from_json(
        contract
            .get_route_requirements(REVOCATION_TEST_CALLER.to_string(), REVOCATION_ROUTE_ID)
            .unwrap()
            .presentation_required,
    )
    .unwrap();

    let revocation_list = req.iter().find(|(k, _)| k == "idx").unwrap();
    assert_eq!(revocation_list.1, Criterion::NotContainedIn(vec![]));

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
            .presentation_required,
    )
    .unwrap();

    let revocation_list = req.iter().find(|(k, _)| k == "idx").unwrap();
    assert_eq!(revocation_list.1, Criterion::NotContainedIn(vec![1, 2, 3]));

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
            .presentation_required,
    )
    .unwrap();

    let revocation_list = req.iter().find(|(k, _)| k == "idx").unwrap();
    assert_eq!(revocation_list.1, Criterion::NotContainedIn(vec![1, 3, 7]));
}

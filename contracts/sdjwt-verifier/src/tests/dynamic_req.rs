use cosmwasm_std::{from_json, to_json_binary, Binary};

use sylvia::multitest::App;

use crate::errors::SdjwtVerifierResultError;
use crate::types::{Criterion, PresentationReq, VerifyResult, IDX};
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use avida_common::types::RegisterRouteRequest;

use super::utils::instantiate_verifier_contract;
use avida_test_utils::sdjwt::{
    claims_with_revocation_idx, make_presentation,
    make_route_verification_requirements_with_req_bytes,
};

#[test]
fn test_dyn_requirements_not_contain() {
    let revoked_idx = 111;
    let dyn_route_id = 123;
    let dyn_app_addr = "dyn_app_addr";
    let caller = "caller_app_addr";

    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    // By default there is no revocation list
    let (contract, _) = instantiate_verifier_contract(&app);

    // Now we create and register a new route with dynamic requirements
    let dyn_presentation_req: PresentationReq = vec![(
        IDX.to_string(),
        Criterion::Dynamic(Box::new(Criterion::NotContainedIn(vec![]))),
    )];

    // Register the app with the two routes
    assert!(contract
        .register(
            dyn_app_addr.to_string(),
            vec![RegisterRouteRequest {
                route_id: dyn_route_id,
                requirements: make_route_verification_requirements_with_req_bytes(
                    serde_json::to_string(&dyn_presentation_req)
                        .unwrap()
                        .as_bytes()
                ),
            }]
        )
        .call(caller)
        .is_ok());

    let dyn_requirement = vec![(
        IDX.to_string(),
        Criterion::NotContainedIn(vec![revoked_idx]),
    )];

    // Make a presentation with some claims
    let revoked_claims = claims_with_revocation_idx("Alice", 30, true, 2021, None, revoked_idx);

    let revoked_presentation = make_presentation(revoked_claims, vec![]);

    // Dyn requirements should be checked if revoked_claims is revoked and should error
    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(revoked_presentation.as_bytes()),
                dyn_route_id,
                Some(dyn_app_addr.to_string()),
                Some(to_json_binary(&dyn_requirement).unwrap()),
            )
            .call(caller)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();
    let err = res.result.unwrap_err();
    assert_eq!(err, SdjwtVerifierResultError::IdxRevoked(revoked_idx));

    // Dyn requirements not present should error
    let res: VerifyResult = from_json(
        contract
            .verify(
                Binary::from(revoked_presentation.as_bytes()),
                dyn_route_id,
                Some(dyn_app_addr.to_string()),
                None,
            )
            .call(caller)
            .unwrap()
            .data
            .unwrap(),
    )
    .unwrap();
    let err = res.result.unwrap_err();
    assert_eq!(err, SdjwtVerifierResultError::DynamicRequirementNotProvided);
}

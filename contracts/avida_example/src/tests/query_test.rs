use crate::constants::GIVE_ME_DRINK_ROUTE_ID;
use crate::msg::ExecuteMsg;
use crate::msg::QueryMsg;
use crate::tests::fixtures::setup_requirement;
use crate::types::RegisterRequirement;
use crate::{tests::fixtures::instantiate_contracts, types::GetVerifierResponse};
use avida_sdjwt_verifier::msg::QueryMsg as VerifierQueryMsg;
use avida_sdjwt_verifier::types::{JwkInfo, VerificationRequirements};
use avida_test_utils::sdjwt::fixtures::OWNER_ADDR;
use cosmwasm_std::{from_json, to_json_binary, Addr};
use cw_multi_test::{App, Executor};
use josekit::jwk::Jwk;
#[test]
fn get_verifier() {
    let mut app = App::default();
    let (restaurant_addr, verifier_addr) = instantiate_contracts(&mut app);

    // Query verifier address
    let response: GetVerifierResponse = app
        .wrap()
        .query_wasm_smart(restaurant_addr, &QueryMsg::GetVerifierAddress {})
        .unwrap();

    assert_eq!(response.verifier, verifier_addr.to_string());
}

#[test]
fn get_route_requirements() {
    let mut app = App::default();

    // Instantiate verifier & restaurant contracts
    let (restaurant_addr, verifier_addr) = instantiate_contracts(&mut app);

    // Setup and register requirement
    let fx_route_verification_req = setup_requirement("drink");
    let register_msg = ExecuteMsg::RegisterRequirement {
        requirements: RegisterRequirement::Drink {
            requirements: fx_route_verification_req.clone(),
        },
    };

    app.execute_contract(
        Addr::unchecked(OWNER_ADDR),
        restaurant_addr.clone(),
        &register_msg,
        &[],
    )
    .unwrap();

    // Query route requirements
    let requirements: VerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            verifier_addr,
            &VerifierQueryMsg::GetRouteRequirements {
                app_addr: restaurant_addr.to_string(),
                route_id: GIVE_ME_DRINK_ROUTE_ID,
            },
        )
        .unwrap();

    let mut registered_pubkeys: Vec<JwkInfo> = requirements
        .issuer_pubkeys
        .unwrap_or_default() // Handle the Option, default to empty HashMap if None
        .into_iter()
        .map(|(iss, key)| JwkInfo {
            issuer: iss,
            jwk: to_json_binary(&key).unwrap(),
        })
        .collect();

    let mut expected_pub_keys: Vec<JwkInfo> = fx_route_verification_req
        .issuer_source_or_data
        .into_iter()
        .map(|isd| from_json::<JwkInfo>(isd.data_or_location).unwrap()) // Consider error handling
        .collect();

    assert_eq!(expected_pub_keys.len(), registered_pubkeys.len());

    // Pop and deserialize for comparison
    let reg_jwk: Jwk = from_json(&registered_pubkeys.pop().unwrap().jwk).unwrap();
    let exp_jwk: Jwk = from_json(&expected_pub_keys.pop().unwrap().jwk).unwrap();
    assert_eq!(reg_jwk, exp_jwk);
}

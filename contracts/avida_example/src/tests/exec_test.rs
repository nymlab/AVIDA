use cosmwasm_std::{from_json, to_json_binary, Addr};
use cw_multi_test::{App, Executor};

use avida_sdjwt_verifier::types::{JwkInfo, VerificationRequirements};
use avida_test_utils::sdjwt::fixtures::{issuer_jwk, OWNER_ADDR};
use josekit::jwk::Jwk;

use crate::constants::GIVE_ME_DRINK_ROUTE_ID;
use crate::msg::ExecuteMsg;
use crate::tests::fixtures::{instantiate_contracts, setup_requirement};
use crate::types::RegisterRequirement;
use avida_sdjwt_verifier::msg::QueryMsg as VerifierQueryMsg;

#[test]
fn register_requirement() {
    let mut app = App::default();

    // Instantiate verifier & restaurant contracts first
    let (restaurant_addr, verifier_addr) = instantiate_contracts(&mut app);

    // Setup requirement
    let fx_route_verification_req = setup_requirement("drink");

    // Register requirement
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

    // Query registered routes from verifier
    let routes: Vec<u64> = app
        .wrap()
        .query_wasm_smart(
            verifier_addr.clone(),
            &VerifierQueryMsg::GetRoutes {
                app_addr: restaurant_addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes.first().unwrap(), &GIVE_ME_DRINK_ROUTE_ID);

    // Query registered requirements
    let registered_req: VerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            verifier_addr.clone(),
            &VerifierQueryMsg::GetRouteRequirements {
                app_addr: restaurant_addr.to_string(),
                route_id: GIVE_ME_DRINK_ROUTE_ID,
            },
        )
        .unwrap();

    let mut registered_pubkeys: Vec<JwkInfo> = registered_req
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

    assert_eq!(
        to_json_binary(&registered_req.presentation_required).unwrap(),
        fx_route_verification_req.presentation_required.unwrap()
    );

    // Query route verification key
    let route_verification_keys: Option<Vec<String>> = app
        .wrap()
        .query_wasm_smart(
            verifier_addr,
            &VerifierQueryMsg::GetRouteVerificationKey {
                app_addr: restaurant_addr.to_string(),
                route_id: GIVE_ME_DRINK_ROUTE_ID,
            },
        )
        .unwrap();

    let rvk = route_verification_keys.unwrap().pop().unwrap();

    let route_verification_jwk: josekit::jwk::Jwk = serde_json::from_str(&rvk).unwrap();
    assert_eq!(route_verification_jwk, issuer_jwk());
}

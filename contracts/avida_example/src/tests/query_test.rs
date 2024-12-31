use crate::constants::GIVE_ME_DRINK_ROUTE_ID;
use crate::msg::ExecuteMsg;
use crate::msg::QueryMsg;
use crate::tests::fixtures::setup_requirement;
use crate::types::RegisterRequirement;
use crate::{tests::fixtures::instantiate_contracts, types::GetVerifierResponse};
use avida_common::types::RouteVerificationRequirements;
use avida_sdjwt_verifier::msg::QueryMsg as VerifierQueryMsg;
use avida_test_utils::sdjwt::fixtures::OWNER_ADDR;
use cosmwasm_std::Addr;
use cw_multi_test::{App, Executor};
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
    let requirements: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            verifier_addr,
            &VerifierQueryMsg::GetRouteRequirements {
                app_addr: restaurant_addr.to_string(),
                route_id: GIVE_ME_DRINK_ROUTE_ID,
            },
        )
        .unwrap();

    // Verify the requirements match what we registered
    assert_eq!(requirements, fx_route_verification_req);
}

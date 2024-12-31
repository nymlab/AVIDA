use cosmwasm_std::Addr;
use cw_multi_test::{App, Executor};

use avida_common::types::RouteVerificationRequirements;
use avida_test_utils::sdjwt::fixtures::{issuer_jwk, OWNER_ADDR};

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
    let registered_req: RouteVerificationRequirements = app
        .wrap()
        .query_wasm_smart(
            verifier_addr.clone(),
            &VerifierQueryMsg::GetRouteRequirements {
                app_addr: restaurant_addr.to_string(),
                route_id: GIVE_ME_DRINK_ROUTE_ID,
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

    // Query route verification key
    let route_verification_key: Option<String> = app
        .wrap()
        .query_wasm_smart(
            verifier_addr,
            &VerifierQueryMsg::GetRouteVerificationKey {
                app_addr: restaurant_addr.to_string(),
                route_id: GIVE_ME_DRINK_ROUTE_ID,
            },
        )
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key.unwrap()).unwrap();
    assert_eq!(route_verification_jwk, issuer_jwk());
}

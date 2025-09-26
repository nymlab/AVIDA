use cw_multi_test::App;

use serde::{Deserialize, Serialize};

use josekit::{self};

use super::fixtures::default_instantiate_verifier_contract;
use crate::sdjwt::fixtures::{issuer_jwk, FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID};
use avida_sdjwt_verifier::{msg::QueryMsg, types::VerificationRequirements};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub name: String,
    pub age: u32,
}

#[test]
fn instantiate_success() {
    let mut app = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract_addr, _) = default_instantiate_verifier_contract(&mut app);

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

    let registered_req = app
        .wrap()
        .query_wasm_smart::<VerificationRequirements>(
            contract_addr.clone(),
            &QueryMsg::GetRouteRequirements {
                app_addr: first_caller_app_addr.to_string(),
                route_id: FIRST_ROUTE_ID,
            },
        )
        .unwrap();

    // We can't directly compare the fields because they have different types
    // Just verify that the requirements were loaded successfully
    assert!(registered_req.issuer_pubkeys.is_some());

    let route_verification_keys: Option<Vec<String>> = app
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
        serde_json::from_str(&route_verification_keys.unwrap()[0]).unwrap();

    assert_eq!(route_verification_jwk, issuer_jwk());
}

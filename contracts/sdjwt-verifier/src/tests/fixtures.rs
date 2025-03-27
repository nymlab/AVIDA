use avida_test_utils::sdjwt::fixtures::{
    get_default_presentation_required, make_route_verification_requirements, ExpirationCheck,
    KeyType, FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID, MAX_PRESENTATION_LEN, OWNER_ADDR,
    VERIFIER_CONTRACT_LABEL,
};

use avida_common::types::{RegisterRouteRequest, RouteVerificationRequirements};

use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App as MtApp, Contract, ContractWrapper, Executor};

use crate::contract;
use crate::msg::InstantiateMsg;
use crate::types::InitRegistration;

fn notarised_odp_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        contract::execute,
        contract::instantiate,
        contract::query,
    ))
}

/// Is used to instantiate verifier contract with some predefined parameters
pub fn default_instantiate_verifier_contract(
    app: &mut MtApp,
) -> (Addr, RouteVerificationRequirements) {
    let presentation_required = get_default_presentation_required(ExpirationCheck::NoExpiry);

    let fx_route_verification_req =
        make_route_verification_requirements(presentation_required, KeyType::Ed25519);

    let contract = notarised_odp_contract();
    let code_id = app.store_code(contract);
    let first_caller_app_addr = app.api().addr_make(FIRST_CALLER_APP_ADDR);
    let init_registrations = vec![InitRegistration {
        app_admin: first_caller_app_addr.to_string(),
        app_addr: first_caller_app_addr.to_string(),
        routes: vec![RegisterRouteRequest {
            route_id: FIRST_ROUTE_ID,
            requirements: fx_route_verification_req.clone(),
        }],
    }];

    let instantiate_msg = InstantiateMsg {
        max_presentation_len: MAX_PRESENTATION_LEN,
        init_registrations,
    };
    let owner = app.api().addr_make(OWNER_ADDR);
    let contract_addr = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            VERIFIER_CONTRACT_LABEL,
            None,
        )
        .unwrap();
    (contract_addr, fx_route_verification_req)
}

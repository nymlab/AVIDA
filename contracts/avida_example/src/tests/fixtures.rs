use avida_common::types::RouteVerificationRequirements;
use avida_sdjwt_verifier::types::{
    Criterion, MathsOperator, NumberCriterion, PresentationReq, ReqAttr, CW_EXPIRATION,
};
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, make_presentation, make_route_verification_requirements,
    KeyType, PresentationVerificationType,
};

use crate::contract;
use crate::msg::InstantiateMsg as RestaurantInstantiateMsg;
use avida_sdjwt_verifier::contract as verifier_contract;
use avida_sdjwt_verifier::msg::InstantiateMsg as VerifierInstantiateMsg;
use avida_test_utils::sdjwt::fixtures::{MAX_PRESENTATION_LEN, OWNER_ADDR};
use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

const VERIFIER_CONTRACT_LABEL: &str = "Verifier";
const RESTAURANT_CONTRACT_LABEL: &str = "Restaurant";

pub fn create_presentation(age: u8) -> String {
    let claims = claims("Alice", age, true, 2021, None);
    make_presentation(claims, PresentationVerificationType::Success)
}

pub fn create_presentation_with_exp(expired: bool) -> String {
    let exp = if expired {
        cw_utils::Expiration::AtTime(get_default_block_info().time.minus_hours(1))
    } else {
        cw_utils::Expiration::AtTime(get_default_block_info().time.plus_hours(1))
    };
    let claims = claims("Alice", 30, true, 2021, Some(exp));
    make_presentation(claims, PresentationVerificationType::Success)
}

pub fn setup_requirement(order: &str) -> RouteVerificationRequirements {
    // Add only 1 criterion - age greater than 18 for drink, and none for food
    let presentation_req: PresentationReq = match order {
        "drink" => vec![ReqAttr {
            attribute: "age".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 18,
                operator: MathsOperator::GreaterThan,
            }),
        }],
        "food" => vec![],
        _ => vec![],
    };
    make_route_verification_requirements(presentation_req, KeyType::Ed25519)
}

pub fn setup_requirement_with_expiration() -> RouteVerificationRequirements {
    // Add 2 criterion - age greater than 18, and expiration
    let presentation_req: PresentationReq = vec![
        ReqAttr {
            attribute: "age".to_string(),
            criterion: Criterion::Number(NumberCriterion {
                value: 18,
                operator: MathsOperator::GreaterThan,
            }),
        },
        ReqAttr {
            attribute: CW_EXPIRATION.to_string(),
            criterion: Criterion::Expires(true),
        },
    ];

    make_route_verification_requirements(presentation_req, KeyType::Ed25519)
}

pub fn verifier_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        verifier_contract::execute,
        verifier_contract::instantiate,
        verifier_contract::query,
    ))
}

pub fn restaurant_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new_with_empty(contract::execute, contract::instantiate, contract::query)
            .with_reply_empty(contract::reply),
    )
}

pub fn instantiate_contracts(app: &mut App) -> (Addr, Addr) {
    // Storages for contracts
    let code_id_verifier = app.store_code(verifier_contract());
    let code_id_restaurant = app.store_code(restaurant_contract());

    // Instantiate contracts
    let verifier_instantiate_msg = VerifierInstantiateMsg {
        max_presentation_len: MAX_PRESENTATION_LEN,
        init_registrations: vec![],
    };

    let caller = app.api().addr_make(OWNER_ADDR);

    let contract_verifier = app
        .instantiate_contract(
            code_id_verifier,
            caller.clone(),
            &verifier_instantiate_msg,
            &[],
            VERIFIER_CONTRACT_LABEL,
            None,
        )
        .unwrap();

    let restaurant_instantiate_msg = RestaurantInstantiateMsg {
        verifier: contract_verifier.to_string(),
    };
    (
        app.instantiate_contract(
            code_id_restaurant,
            caller,
            &restaurant_instantiate_msg,
            &[],
            RESTAURANT_CONTRACT_LABEL,
            None,
        )
        .unwrap(),
        contract_verifier,
    )
}

use avida_common::types::RouteVerificationRequirements;
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq, CW_EXPIRATION};
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, make_presentation, make_route_verification_requirements,
};

use crate::contract::{sv::mt::CodeId as RestaurantCodeID, RestaurantContract};
use avida_sdjwt_verifier::contract::sv::mt::CodeId as VerifierCodeID;
use avida_test_utils::sdjwt::fixtures::{MAX_PRESENTATION_LEN, OWNER_ADDR as caller};
use sylvia::cw_multi_test::App as MtApp;
use sylvia::multitest::{App, Proxy};

pub fn create_presentation(age: u8) -> String {
    let claims = claims("Alice", age, true, 2021, None);
    make_presentation(claims, vec![])
}

pub fn create_presentation_with_exp(expired: bool) -> String {
    let exp = if expired {
        cw_utils::Expiration::AtTime(get_default_block_info().time.minus_hours(1))
    } else {
        cw_utils::Expiration::AtTime(get_default_block_info().time.plus_hours(1))
    };
    let claims = claims("Alice", 30, true, 2021, Some(exp));
    make_presentation(claims, vec![])
}

pub fn setup_requirement(order: &str) -> RouteVerificationRequirements {
    // Add only 1 criterion - age greater than 18 for drink, and none for food
    let presentation_req: PresentationReq = match order {
        "drink" => vec![(
            "age".to_string(),
            Criterion::Number(18, MathsOperator::GreaterThan),
        )],
        "food" => vec![],
        _ => vec![],
    };
    make_route_verification_requirements(presentation_req)
}

pub fn setup_requirement_with_expiration() -> RouteVerificationRequirements {
    // Add 2 criterion - age greater than 18, and expiration
    let presentation_req: PresentationReq = vec![
        (
            "age".to_string(),
            Criterion::Number(18, MathsOperator::GreaterThan),
        ),
        (CW_EXPIRATION.to_string(), Criterion::Expires(true)),
    ];

    make_route_verification_requirements(presentation_req)
}

pub fn instantiate_contracts(app: &App<MtApp>) -> Proxy<'_, MtApp, RestaurantContract<'_>> {
    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(app);
    let code_id_restaurant = RestaurantCodeID::store_code(app);

    // Instantiate contracts
    let contract_verifier = code_id_verifier
        .instantiate(MAX_PRESENTATION_LEN, vec![])
        .with_label("Verifier")
        .call(caller)
        .unwrap();

    code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(caller)
        .unwrap()
}

use cosmwasm_std::Binary;
use sylvia::multitest::App;

use crate::contract::sv::mt::RestaurantContractProxy;
use avida_test_utils::sdjwt::fixtures::OWNER_ADDR as caller;

use crate::tests::fixtures::{
    create_presentation, create_presentation_with_exp, instantiate_contracts, setup_requirement,
    setup_requirement_with_expiration,
};
use crate::types::{GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};

#[test]
pub fn flow_drink_verification() {
    let app = App::default();
    let contract_restaurant = instantiate_contracts(&app);
    // Setup requirement
    let fx_route_verification_req = setup_requirement("drink");
    contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req,
        })
        .call(caller)
        .unwrap();

    let presentation = create_presentation(30);

    let msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_drink(msg)
        .call(caller)
        .unwrap();
    // Check that there is an event with key-value
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .clone()
            .into_iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
            && event
                .attributes
                .into_iter()
                .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));

    let presentation = create_presentation(30);

    let msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_drink(msg)
        .call(caller)
        .unwrap();
    // Check that there is an event with key-value
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .clone()
            .into_iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
            && event
                .attributes
                .into_iter()
                .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));
}

#[test]
#[should_panic]
pub fn flow_drink_verification_underage_fails() {
    let app = App::default();
    let contract_restaurant = instantiate_contracts(&app);
    // Setup requirement
    let fx_route_verification_req = setup_requirement("drink");
    contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req,
        })
        .call(caller)
        .unwrap();

    let presentation = create_presentation(10);

    let msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let _ = contract_restaurant.give_me_some_drink(msg).call(caller);
}

#[test]
pub fn flow_food_verification() {
    let app = App::default();
    let contract_restaurant = instantiate_contracts(&app);

    // Setup requirement
    let fx_route_verification_req = setup_requirement("food");
    contract_restaurant
        .register_requirement(RegisterRequirement::Food {
            requirements: fx_route_verification_req.clone(),
        })
        .call(caller)
        .unwrap();

    let presentation = create_presentation(11);

    let msg = GiveMeSomeFood {
        kind: "Gazpacho".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_food(msg)
        .call(caller)
        .unwrap();

    // Check that there is an event with key-value
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .clone()
            .into_iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_food")
            && event
                .attributes
                .into_iter()
                .any(|attr| attr.key == "Food kind" && attr.value == "Gazpacho")
    }));
}

#[test]
pub fn flow_drink_not_expired_verification() {
    let app = App::default();
    let contract_restaurant = instantiate_contracts(&app);

    // Setup requirement
    let fx_route_verification_req_with_expiration = setup_requirement_with_expiration();
    contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req_with_expiration.clone(),
        })
        .call(caller)
        .unwrap();

    let valid_presentation = create_presentation_with_exp(false);

    let valid_msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(valid_presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_drink(valid_msg)
        .call(caller);

    assert!(resp.is_ok());
    let events = resp.unwrap().events;
    // Check that there is an event with key-value
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(events.into_iter().any(|event| {
        event
            .attributes
            .clone()
            .into_iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
            && event
                .attributes
                .into_iter()
                .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));
}

#[test]
#[should_panic]
pub fn flow_drink_expired_verification() {
    let app = App::default();
    let contract_restaurant = instantiate_contracts(&app);

    // Setup requirement
    let fx_route_verification_req_with_expiration = setup_requirement_with_expiration();
    contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req_with_expiration.clone(),
        })
        .call(caller)
        .unwrap();

    let invalid_presentation = create_presentation_with_exp(true);

    let invalid_msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(invalid_presentation.as_bytes()),
    };
    // This panics becasue error downcasting failed
    let _ = contract_restaurant
        .give_me_some_drink(invalid_msg)
        .call(caller);
}

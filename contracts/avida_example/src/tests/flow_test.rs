use cosmwasm_std::{Addr, Binary};
use sylvia::multitest::App;

use avida_sdjwt_verifier::contract::sv::mt::CodeId as VerifierCodeID;

use crate::contract::sv::mt::{CodeId as RestaurantCodeID, RestaurantContractProxy};
use crate::tests::fixtures::{
    create_presentation, create_presentation_with_exp, setup_requirement,
    setup_requirement_with_expiration,
};
use crate::types::{GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};

#[test]
pub fn flow_drink_verification() {
    let app = App::default();

    let owner = Addr::unchecked("owner"); // "owner";
    let caller = Addr::unchecked("caller"); // "caller";

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(owner.as_str())
        .unwrap();

    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(owner.as_str())
        .unwrap();

    // Setup requirement
    let fx_route_verification_req = setup_requirement();
    let _a = contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req.clone(),
        })
        .call(owner.as_str())
        .unwrap();

    let presentation = create_presentation();

    let msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_drink(msg)
        .call(caller.as_str());

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
pub fn flow_food_verification() {
    let app = App::default();

    let owner = Addr::unchecked("owner"); // "owner";
    let caller = Addr::unchecked("caller"); // "caller";

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(owner.as_str())
        .unwrap();

    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(owner.as_str())
        .unwrap();

    // Setup requirement
    let fx_route_verification_req = setup_requirement();
    let _a = contract_restaurant
        .register_requirement(RegisterRequirement::Food {
            requirements: fx_route_verification_req.clone(),
        })
        .call(owner.as_str())
        .unwrap();

    let presentation = create_presentation();

    let msg = GiveMeSomeFood {
        kind: "Gazpacho".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_food(msg)
        .call(caller.as_str());

    assert!(resp.is_ok());
    let events = resp.unwrap().events;
    // Check that there is an event with key-value
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(events.into_iter().any(|event| {
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

    let owner = Addr::unchecked("owner"); // "owner";
    let caller = Addr::unchecked("caller"); // "caller";

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(owner.as_str())
        .unwrap();

    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(owner.as_str())
        .unwrap();

    // Setup requirement
    let fx_route_verification_req_with_expiration = setup_requirement_with_expiration();
    let _a = contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req_with_expiration.clone(),
        })
        .call(owner.as_str())
        .unwrap();

    let valid_presentation = create_presentation_with_exp(false);

    let valid_msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(valid_presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_drink(valid_msg)
        .call(caller.as_str());

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

    let owner = Addr::unchecked("owner"); // "owner";
    let caller = Addr::unchecked("caller"); // "caller";

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(owner.as_str())
        .unwrap();

    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(owner.as_str())
        .unwrap();

    // Setup requirement
    let fx_route_verification_req_with_expiration = setup_requirement_with_expiration();
    let _a = contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req_with_expiration.clone(),
        })
        .call(owner.as_str())
        .unwrap();

    let invalid_presentation = create_presentation_with_exp(true);

    let invalid_msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(invalid_presentation.as_bytes()),
    };
    // This panics becasue error downcasting failed
    let _ = contract_restaurant
        .give_me_some_drink(invalid_msg)
        .call(caller.as_str());
}

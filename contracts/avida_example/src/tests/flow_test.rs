use cosmwasm_std::{Addr, Binary};
use sylvia::multitest::App;

use avida_common::types::RouteVerificationRequirements;
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use avida_sdjwt_verifier::contract::sv::mt::{CodeId as VerifierCodeID, SdjwtVerifierProxy};

use crate::constants::GIVE_ME_DRINK_ROUTE_ID;
use crate::msg::{GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};
use crate::contract::sv::mt::{CodeId as RestaurantCodeID, RestaurantContractProxy};
use crate::tests::fixtures::{create_presentation, setup_requirement};
use super::fixtures::issuer_jwk;



#[test]
fn register_requirement() {
    let app = App::default();
    let owner = Addr::unchecked("owner"); // "owner";
    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(&owner.as_str())
        .unwrap();
    
    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(&owner.as_str())
        .unwrap();
    // Setup requirement
    let fx_route_verification_req = setup_requirement();
    let _a = contract_restaurant
        .register_requirement(
            RegisterRequirement::Drink { 
                requirements: fx_route_verification_req.clone() 
            })
        .call(&owner.as_str())
        .unwrap();
    let registered_routes = contract_verifier.get_routes(contract_restaurant.contract_addr.to_string()).unwrap();

    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &GIVE_ME_DRINK_ROUTE_ID);

    let registered_req = contract_verifier
        .get_route_requirements(contract_restaurant.contract_addr.to_string(), GIVE_ME_DRINK_ROUTE_ID)
        .unwrap();

    assert_eq!(
        registered_req.verification_source,
        fx_route_verification_req.verification_source
    );

    assert_eq!(
        registered_req.presentation_request,
        fx_route_verification_req.presentation_request
    );

    let route_verification_key = contract_verifier
        .get_route_verification_key(contract_restaurant.contract_addr.to_string(), GIVE_ME_DRINK_ROUTE_ID)
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();
    assert_eq!(route_verification_jwk, issuer_jwk());
}

#[test]
pub fn flow_drink_verification() {
    let app = App::default();

    let owner = Addr::unchecked("owner"); // "owner";
    let caller = Addr::unchecked("caller"); // "caller";
    let fx_route_verification_req: RouteVerificationRequirements;

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(&owner.as_str())
        .unwrap();
    
    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(&owner.as_str())
        .unwrap();
    
    // Setup requirement
    fx_route_verification_req = setup_requirement();
    let _a = contract_restaurant
        .register_requirement(
            RegisterRequirement::Drink { 
                requirements: fx_route_verification_req.clone() 
            })
        .call(&owner.as_str())
        .unwrap();

    let presentation = create_presentation();

    let msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_drink(
            msg
        )
        .call(&caller.as_str());
    
    assert!(resp.is_ok());
    let events = resp.unwrap().events;
    // Check that there is an event with key-value 
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(events.into_iter().any(|event| {
        event.attributes.clone()
            .into_iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
        &&
        event.attributes
            .into_iter()
            .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));
}

#[test]
pub fn flow_food_verification() {
    let app = App::default();

    let owner = Addr::unchecked("owner"); // "owner";
    let caller = Addr::unchecked("caller"); // "caller";
    let fx_route_verification_req: RouteVerificationRequirements;

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .call(&owner.as_str())
        .unwrap();
    
    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(&owner.as_str())
        .unwrap();
    
    // Setup requirement
    fx_route_verification_req = setup_requirement();
    let _a = contract_restaurant
        .register_requirement(
            RegisterRequirement::Food { 
                requirements: fx_route_verification_req.clone() 
            })
        .call(&owner.as_str())
        .unwrap();

    let presentation = create_presentation();

    let msg = GiveMeSomeFood {
        kind: "Gazpacho".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let resp = contract_restaurant
        .give_me_some_food(
            msg
        )
        .call(&caller.as_str());
    
    assert!(resp.is_ok());
    let events = resp.unwrap().events;
    // Check that there is an event with key-value 
    // {"action": "give_me_some_drink"} and {"drink": "beer"}
    assert!(events.into_iter().any(|event| {
        event.attributes.clone()
            .into_iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_food")
        &&
        event.attributes
            .into_iter()
            .any(|attr| attr.key == "Food kind" && attr.value == "Gazpacho")
    }));
}
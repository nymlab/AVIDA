use cosmwasm_std::Addr;
use sylvia::multitest::App;

use avida_test_utils::sdjwt::fixtures::OWNER_ADDR;

use crate::contract::sv::mt::{CodeId as RestaurantCodeID, RestaurantContractProxy};

#[test]
fn get_verifier() {
    let app = App::default();
    let verifier_contract_addr = Addr::unchecked("verifier"); // "verifier";
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    let contract_restaurant = code_id_restaurant
        .instantiate(verifier_contract_addr.to_string())
        .with_label("Restaurant")
        .call(OWNER_ADDR)
        .unwrap();

    let asked_verifier = contract_restaurant.get_verifier_address().unwrap();
    assert_eq!(asked_verifier.verifier, verifier_contract_addr);
}

//#[test]
//fn get_route_requirements() {
//    let app = App::default();
//    // Storages for contracts
//    let code_id_verifier = VerifierCodeID::store_code(&app);
//    let code_id_restaurant = RestaurantCodeID::store_code(&app);
//
//    // Instantiate contracts
//    let max_presentation_len = 3000usize;
//    let contract_verifier = code_id_verifier
//        .instantiate(max_presentation_len, vec![])
//        .with_label("Verifier")
//        .call(OWNER_ADDR)
//        .unwrap();
//
//    let contract_restaurant = code_id_restaurant
//        .instantiate(contract_verifier.contract_addr.to_string())
//        .with_label("Restaurant")
//        .call(OWNER_ADDR)
//        .unwrap();
//    // Setup requirement
//    let fx_route_verification_req = setup_requirement("drink");
//    contract_restaurant
//        .register_requirement(RegisterRequirement::Drink {
//            requirements: fx_route_verification_req.clone(),
//        })
//        .call(OWNER_ADDR)
//        .unwrap();
//    let registered_routes = contract_restaurant
//        .get_route_requirements(GIVE_ME_DRINK_ROUTE_ID)
//        .unwrap();
//
//    assert_eq!(registered_routes, fx_route_verification_req);
//}

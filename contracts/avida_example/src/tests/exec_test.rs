use sylvia::multitest::App;

use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use avida_sdjwt_verifier::contract::sv::mt::{CodeId as VerifierCodeID, SdjwtVerifierProxy};

use crate::constants::GIVE_ME_DRINK_ROUTE_ID;
use crate::contract::sv::mt::{CodeId as RestaurantCodeID, RestaurantContractProxy};
use crate::tests::fixtures::setup_requirement;
use crate::types::RegisterRequirement;
use avida_test_utils::sdjwt::fixtures::{issuer_jwk, MAX_PRESENTATION_LEN, OWNER_ADDR};

#[test]
fn register_requirement() {
    let app = App::default();
    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let contract_verifier = code_id_verifier
        .instantiate(MAX_PRESENTATION_LEN, vec![])
        .with_label("Verifier")
        .call(OWNER_ADDR)
        .unwrap();

    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .call(OWNER_ADDR)
        .unwrap();
    // Setup requirement
    let fx_route_verification_req = setup_requirement("drink");
    let _a = contract_restaurant
        .register_requirement(RegisterRequirement::Drink {
            requirements: fx_route_verification_req.clone(),
        })
        .call(OWNER_ADDR)
        .unwrap();
    let registered_routes = contract_verifier
        .get_routes(contract_restaurant.contract_addr.to_string())
        .unwrap();

    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &GIVE_ME_DRINK_ROUTE_ID);

    let registered_req = contract_verifier
        .get_route_requirements(
            contract_restaurant.contract_addr.to_string(),
            GIVE_ME_DRINK_ROUTE_ID,
        )
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
        .get_route_verification_key(
            contract_restaurant.contract_addr.to_string(),
            GIVE_ME_DRINK_ROUTE_ID,
        )
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();
    assert_eq!(route_verification_jwk, issuer_jwk());
}

use sylvia::multitest::App;

use crate::errors::SdjwtVerifierError;
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;

use super::utils::instantiate_verifier_contract;
use avida_test_utils::sdjwt::{
    get_two_input_routes_requirements, OWNER_ADDR, SECOND_CALLER_APP_ADDR,
};

#[test]
fn deregister_success() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Unregister the app
    assert!(contract
        .deregister(SECOND_CALLER_APP_ADDR.to_string())
        .call(OWNER_ADDR)
        .is_ok());

    // Ensure there is no routes left after the app deregistration
    let registered_routes = contract.get_routes(SECOND_CALLER_APP_ADDR.to_string());

    assert!(registered_routes.is_err());
}

#[test]
fn deregister_app_not_registered() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Try deregister the not registered app
    assert!(matches!(
        contract
            .deregister(SECOND_CALLER_APP_ADDR.to_string(),)
            .call(OWNER_ADDR),
        Err(SdjwtVerifierError::AppIsNotRegistered)
    ),);
}

#[test]
fn deregister_unathorized() {
    let app: App<_> = App::default();

    // Instantiate verifier contract with some predefined parameters
    let (contract, _) = instantiate_verifier_contract(&app);

    // Get input verification requirements for 2 routes
    let two_routes_verification_req = get_two_input_routes_requirements();

    // Register the app with the two routes
    assert!(contract
        .register(
            SECOND_CALLER_APP_ADDR.to_string(),
            two_routes_verification_req
        )
        .call(OWNER_ADDR)
        .is_ok());

    // Try deregister the app using unathorized caller address
    assert!(matches!(
        contract
            .deregister(SECOND_CALLER_APP_ADDR.to_string(),)
            .call(SECOND_CALLER_APP_ADDR),
        Err(SdjwtVerifierError::Unauthorised)
    ),);
}

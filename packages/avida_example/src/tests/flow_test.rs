use avida_common::types::{InputRoutesRequirements, RouteVerificationRequirements, VerificationSource};
use avida_common::traits::avida_verifier_trait::sv::mt::AvidaVerifierTraitProxy;
use cosmwasm_std::Binary;

use sd_jwt_rs::issuer;
use sd_jwt_rs::{SDJWTHolder, SDJWTSerializationFormat};

use sylvia::multitest::App;
use crate::constants::GIVE_ME_DRINK_ROUTE_ID;
use crate::contract::sv::ExecMsg;
use crate::msg::{GiveMeSomeDrink, RegisterRequirement};
use crate::contract::sv::mt::{CodeId as RestaurantCodeID, RestaurantContractProxy};
use avida_sdjwt_verifier::contract::sv::mt::{CodeId as VerifierCodeID, SdjwtVerifierProxy};
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq};
use super::fixtures::{claims, issuer, issuer_jwk};
use josekit::{self, Value};

#[test]
pub fn flow_drink_verification() {
    let app = App::default();

    let verifier_owner = "addr0001";
    let restaurant_owner = "addr0002";
    let caller = "addr0003";
    let mut fx_issuer = issuer();

    // Add only 1 criterion - age greater than 18
    let presentation_req: PresentationReq = vec![
        (
            "age".to_string(),
            Criterion::Number(18, MathsOperator::GreaterThan),
        )
    ];

    let request_serialized = serde_json::to_string(&presentation_req).unwrap();
    let fx_jwk = serde_json::to_string(&issuer_jwk()).unwrap();

    println!("fx_jwk: {:#?}", fx_jwk);

    // Add some default criteria as presentation request
    let fx_route_verification_req: RouteVerificationRequirements = RouteVerificationRequirements {
        verification_source: VerificationSource {
            source: None,
            data_or_location: Binary::from(fx_jwk.as_bytes()),
        },
        presentation_request: Binary::from(request_serialized.as_bytes()),
    };

    // Storages for contracts
    let code_id_verifier = VerifierCodeID::store_code(&app);
    let code_id_restaurant = RestaurantCodeID::store_code(&app);

    // Instantiate contracts
    let max_presentation_len = 3000usize;
    let contract_verifier = code_id_verifier
        .instantiate(max_presentation_len, vec![])
        .with_label("Verifier")
        .with_admin(verifier_owner)
        .call(&verifier_owner)
        .unwrap();
    
    let contract_restaurant = code_id_restaurant
        .instantiate(contract_verifier.contract_addr.to_string())
        .with_label("Restaurant")
        .with_admin(restaurant_owner)
        .call(&restaurant_owner)
        .unwrap();

    // Register route to verifier
    let _ = contract_verifier
        .register(
            caller.to_string(),
            vec![InputRoutesRequirements{
                route_id: GIVE_ME_DRINK_ROUTE_ID,
                requirements: fx_route_verification_req.clone()
            }]
        )
    .call(&caller)
    .unwrap();
    // let _a = contract_restaurant
    //     .register_requirement(
    //         RegisterRequirement::Drink { requirements: fx_route_verification_req.clone() },
    //     )
    //     .call(&caller);
    // Check that verifier has route registered.
    let registered_routes = contract_verifier.get_routes(caller.to_string()).unwrap();

    assert_eq!(registered_routes.len(), 1);
    assert_eq!(registered_routes.first().unwrap(), &GIVE_ME_DRINK_ROUTE_ID);

    let registered_req = contract_verifier
        .get_route_requirements(caller.to_string(), GIVE_ME_DRINK_ROUTE_ID)
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
        .get_route_verification_key(caller.to_string(), GIVE_ME_DRINK_ROUTE_ID)
        .unwrap()
        .unwrap();

    let route_verification_jwk: josekit::jwk::Jwk =
        serde_json::from_str(&route_verification_key).unwrap();
    assert_eq!(route_verification_jwk, issuer_jwk());


    let claims = claims("Alice", 30, true, 2021);
    let sdjwt = fx_issuer
        .issue_sd_jwt(
            claims.clone(),
            issuer::ClaimsForSelectiveDisclosureStrategy::AllLevels,
            None,
            false,
            SDJWTSerializationFormat::Compact,
        )
        .unwrap();

    let mut claims_to_disclosure = claims.clone();
    claims_to_disclosure["name"] = Value::Bool(false);
    claims_to_disclosure["age"] = Value::Bool(true);
    claims_to_disclosure["active"] = Value::Bool(true);
    claims_to_disclosure["joined_at"] = Value::Bool(true);
    let c = claims_to_disclosure.as_object().unwrap().clone();

    let mut holder = SDJWTHolder::new(sdjwt, SDJWTSerializationFormat::Compact).unwrap();
    let presentation = holder
        .create_presentation(c, None, None, None, None)
        .unwrap();

    let msg = GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    };
    let _aa = contract_restaurant
        .give_me_some_drink(
            msg
        )
        .call(&caller)
        .unwrap();
    
    println!("aa: {:#?}", _aa);
}
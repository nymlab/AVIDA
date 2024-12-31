use cosmwasm_std::{Addr, Binary};
use cw_multi_test::{App, Executor};

use crate::msg::ExecuteMsg;
use crate::tests::fixtures::{
    create_presentation, create_presentation_with_exp, instantiate_contracts, setup_requirement,
    setup_requirement_with_expiration,
};
use crate::types::{GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};
use avida_test_utils::sdjwt::fixtures::OWNER_ADDR;

#[test]
fn flow_drink_verification() {
    let mut app = App::default();

    // Setup contracts using the helper function
    let (restaurant_addr, _verifier_addr) = instantiate_contracts(&mut app);

    // Setup requirement
    let fx_route_verification_req = setup_requirement("drink");

    // Register requirement
    let register_msg = ExecuteMsg::RegisterRequirement {
        requirements: RegisterRequirement::Drink {
            requirements: fx_route_verification_req,
        },
    };

    app.execute_contract(
        Addr::unchecked(OWNER_ADDR),
        restaurant_addr.clone(),
        &register_msg,
        &[],
    )
    .unwrap();

    // Create and send drink request with presentation
    let presentation = create_presentation(30);
    let msg = ExecuteMsg::GiveMeSomeDrink(GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    });

    let resp = app
        .execute_contract(
            Addr::unchecked(OWNER_ADDR),
            restaurant_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Verify events
    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
            && event
                .attributes
                .iter()
                .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));

    // Test second drink request
    let presentation = create_presentation(30);
    let msg = ExecuteMsg::GiveMeSomeDrink(GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    });

    let resp = app
        .execute_contract(Addr::unchecked(OWNER_ADDR), restaurant_addr, &msg, &[])
        .unwrap();

    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
            && event
                .attributes
                .iter()
                .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));
}

#[test]
#[should_panic]
fn flow_drink_verification_underage_fails() {
    let mut app = App::default();
    let (restaurant_addr, _) = instantiate_contracts(&mut app);

    // Setup requirement
    let fx_route_verification_req = setup_requirement("drink");
    let register_msg = ExecuteMsg::RegisterRequirement {
        requirements: RegisterRequirement::Drink {
            requirements: fx_route_verification_req,
        },
    };

    app.execute_contract(
        Addr::unchecked(OWNER_ADDR),
        restaurant_addr.clone(),
        &register_msg,
        &[],
    )
    .unwrap();

    // Create and send drink request with underage presentation
    let presentation = create_presentation(10); // Age 10 - underage
    let msg = ExecuteMsg::GiveMeSomeDrink(GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    });

    // This should panic due to age verification failure
    app.execute_contract(Addr::unchecked(OWNER_ADDR), restaurant_addr, &msg, &[])
        .unwrap();
}

#[test]
fn flow_food_verification() {
    let mut app = App::default();
    let (restaurant_addr, _) = instantiate_contracts(&mut app);

    // Setup requirement
    let fx_route_verification_req = setup_requirement("food");
    let register_msg = ExecuteMsg::RegisterRequirement {
        requirements: RegisterRequirement::Food {
            requirements: fx_route_verification_req,
        },
    };

    app.execute_contract(
        Addr::unchecked(OWNER_ADDR),
        restaurant_addr.clone(),
        &register_msg,
        &[],
    )
    .unwrap();

    // Create and send food request
    let presentation = create_presentation(11);
    let msg = ExecuteMsg::GiveMeSomeFood(GiveMeSomeFood {
        kind: "Gazpacho".to_string(),
        proof: Binary::from(presentation.as_bytes()),
    });

    let resp = app
        .execute_contract(Addr::unchecked(OWNER_ADDR), restaurant_addr, &msg, &[])
        .unwrap();

    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_food")
            && event
                .attributes
                .iter()
                .any(|attr| attr.key == "Food kind" && attr.value == "Gazpacho")
    }));
}

#[test]
fn flow_drink_not_expired_verification() {
    let mut app = App::default();
    let (restaurant_addr, _) = instantiate_contracts(&mut app);

    // Setup requirement with expiration
    let fx_route_verification_req = setup_requirement_with_expiration();
    let register_msg = ExecuteMsg::RegisterRequirement {
        requirements: RegisterRequirement::Drink {
            requirements: fx_route_verification_req,
        },
    };

    app.execute_contract(
        Addr::unchecked(OWNER_ADDR),
        restaurant_addr.clone(),
        &register_msg,
        &[],
    )
    .unwrap();

    // Create and send drink request with valid expiration
    let valid_presentation = create_presentation_with_exp(false);
    let msg = ExecuteMsg::GiveMeSomeDrink(GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(valid_presentation.as_bytes()),
    });

    let resp = app
        .execute_contract(Addr::unchecked(OWNER_ADDR), restaurant_addr, &msg, &[])
        .unwrap();

    assert!(resp.events.into_iter().any(|event| {
        event
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "give_me_some_drink")
            && event
                .attributes
                .iter()
                .any(|attr| attr.key == "Drink kind" && attr.value == "beer")
    }));
}

#[test]
#[should_panic]
fn flow_drink_expired_verification() {
    let mut app = App::default();
    let (restaurant_addr, _) = instantiate_contracts(&mut app);

    // Setup requirement with expiration
    let fx_route_verification_req = setup_requirement_with_expiration();
    let register_msg = ExecuteMsg::RegisterRequirement {
        requirements: RegisterRequirement::Drink {
            requirements: fx_route_verification_req,
        },
    };

    app.execute_contract(
        Addr::unchecked(OWNER_ADDR),
        restaurant_addr.clone(),
        &register_msg,
        &[],
    )
    .unwrap();

    // Create and send drink request with expired presentation
    let invalid_presentation = create_presentation_with_exp(true); // expired = true
    let msg = ExecuteMsg::GiveMeSomeDrink(GiveMeSomeDrink {
        kind: "beer".to_string(),
        proof: Binary::from(invalid_presentation.as_bytes()),
    });

    // This should panic due to expiration
    app.execute_contract(Addr::unchecked(OWNER_ADDR), restaurant_addr, &msg, &[])
        .unwrap();
}

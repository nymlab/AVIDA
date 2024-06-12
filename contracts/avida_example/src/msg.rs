use cosmwasm_schema::cw_serde;
use avida_common::types::{ RouteVerificationRequirements, VerfiablePresentation};

#[cw_serde]
pub struct InstantiateMsg {
    // Contract address where verifier is
    verifier: String
}

#[cw_serde]
pub enum RegisterRequirement {
    Drink {requirements: RouteVerificationRequirements},
    Food {requirements: RouteVerificationRequirements},
}

#[cw_serde]
pub struct GiveMeSomeDrink {
    pub kind: String,
    pub proof: VerfiablePresentation,
}

#[cw_serde]
pub struct GiveMeSomeFood {
    pub kind: String,
    pub proof: VerfiablePresentation,
}

#[cw_serde]
pub enum ExecuteMsg {
    GiveMeSomeDrink(GiveMeSomeDrink),
    GiveMeSomeFood(GiveMeSomeFood),
}

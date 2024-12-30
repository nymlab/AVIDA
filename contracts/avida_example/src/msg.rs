use cosmwasm_schema::cw_serde;

use crate::types::{GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};

#[cw_serde]
pub struct InstantiateMsg {
    pub verifier: String,
}

pub enum ExecuteMsg {
    RegisterRequirement { requirements: RegisterRequirement },
    GiveMeSomeDrink(GiveMeSomeDrink),
    GiveMeSomeFood(GiveMeSomeFood),
}

#[cw_serde]
pub enum QueryMsg {
    GetVerifierAddress,
}

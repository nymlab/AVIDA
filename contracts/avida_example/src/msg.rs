use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::{GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};

#[cw_serde]
pub struct InstantiateMsg {
    pub verifier: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterRequirement { requirements: RegisterRequirement },
    GiveMeSomeDrink(GiveMeSomeDrink),
    GiveMeSomeFood(GiveMeSomeFood),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetVerifierAddress,
}

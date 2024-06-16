use avida_common::types::{RouteVerificationRequirements, VerfiablePresentation};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    // Contract address where verifier is
    verifier: String,
}

#[cw_serde]
pub enum RegisterRequirement {
    Drink {
        requirements: RouteVerificationRequirements,
    },
    Food {
        requirements: RouteVerificationRequirements,
    },
}

pub type OrderSubject = String;

#[cw_serde]
pub struct GiveMeSomeDrink {
    pub kind: OrderSubject,
    pub proof: VerfiablePresentation,
}

#[cw_serde]
pub struct GiveMeSomeFood {
    pub kind: OrderSubject,
    pub proof: VerfiablePresentation,
}

// Query messages
#[cw_serde]
pub struct GetVerifierResponse {
    pub verifier: String,
}

#[cw_serde]
pub struct GetRegisteredRequirementResponse {
    pub requirements: RouteVerificationRequirements,
}

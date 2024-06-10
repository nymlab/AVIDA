use cosmwasm_schema::cw_serde;
use avida_common::types::{ InputRoutesRequirements, RouteId, RouteVerificationRequirements, VerfiablePresentation};
use cosmwasm_std::{to_json_binary, CosmosMsg, StdResult, Uint64, WasmMsg};

#[cw_serde]
pub struct InstantiateMsg {
    // Contract address where verifier is
    verifier: String
}

#[cw_serde]
pub enum RegisterRequirement {
    Drink {requirements: RouteVerificationRequirements},
    Food {requirements: RouteVerificationRequirements},
    Gasoline {requirements: RouteVerificationRequirements},
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
pub struct GiveMeSomeGasoline {
    pub amount: Uint64,
    pub proof: VerfiablePresentation,
}

#[cw_serde]
pub enum ExecuteMsg {
    GiveMeSomeDrink(GiveMeSomeDrink),
    GiveMeSomeFood(GiveMeSomeFood),
    GiveMeSomeGasoline(GiveMeSomeGasoline),
}

#[cw_serde]
pub struct RegisterRequest {
    pub app_addr: String,
    pub route_criteria: Vec<InputRoutesRequirements>,
}

#[cw_serde]
pub struct VerifyRequest {
    pub presentation: VerfiablePresentation,
    pub route_id: RouteId,
    pub app_addr: Option<String>,
}

// IntoCosmos trait
pub trait IntoCosmos {
    fn into_cosmos_msg(&self, contract_addr: String) -> StdResult<CosmosMsg>;
}

macro_rules! into_cosmos_msg {
    () => {
        fn into_cosmos_msg(&self, contract_addr: String) -> StdResult<CosmosMsg> {
            let msg = to_json_binary(self)?;
            let execute_msg = WasmMsg::Execute {
                contract_addr: contract_addr.into(),
                msg,
                funds: vec![],
            };
            Ok(execute_msg.into())
        }
    }
}

impl IntoCosmos for RegisterRequest {
    into_cosmos_msg!();
}

impl IntoCosmos for VerifyRequest {
    into_cosmos_msg!();
}

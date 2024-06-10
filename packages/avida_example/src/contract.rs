use avida_common::types::InputRoutesRequirements;
use cosmwasm_std::{entry_point, from_json, DepsMut, Env, Reply, Response, StdResult, SubMsg};
use sylvia::{
    contract, entry_points, schemars,
    types::InstantiateCtx,
};
use cw_storage_plus::{Item};
use cw_utils::parse_reply_execute_data;

use crate::error::ContractError;
use crate::msg::{IntoCosmos, VerifyRequest};
use crate::constants::{GIVE_ME_DRINK_ROUTE_ID, GIVE_ME_FOOD_ROUTE_ID, GIVE_ME_GASOLINE_ROUTE_ID, REGISTER_REQUIREMENT_REPLY_ID};
use crate::msg::{ExecuteMsg, GiveMeSomeDrink, GiveMeSomeFood, GiveMeSomeGasoline, RegisterRequirement, RegisterRequest};
use crate::state::PENDING_TRANSACTIONS;


pub struct RestaurantContract <'a>{
    verifier: Item<'a, String>,
}

#[entry_points]
#[contract]
impl RestaurantContract <'_> {
    pub const fn new() -> Self {
        Self {
            verifier: Item::new("verifier"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx, verifier: String) -> StdResult<Response> {
        let InstantiateCtx { deps, .. } = ctx;
        self.verifier.save(deps.storage, &verifier)?;
        Ok(Response::default())
    }

    // Register the permission policy
    #[sv::msg(exec)]
    pub fn register_requirement(&self, ctx: sylvia::types::ExecCtx, msg: RegisterRequirement) -> StdResult<Response> {
        let route_requirements: InputRoutesRequirements;
        match msg {
            RegisterRequirement::Drink { requirements } => {
                route_requirements = InputRoutesRequirements{
                    route_id: GIVE_ME_DRINK_ROUTE_ID,
                    requirements: requirements
                }
            }
            RegisterRequirement::Food { requirements } => {
                route_requirements = InputRoutesRequirements{
                    route_id: GIVE_ME_FOOD_ROUTE_ID,
                    requirements: requirements
                }
            }
            RegisterRequirement::Gasoline { requirements } => {
                route_requirements = InputRoutesRequirements{
                    route_id: GIVE_ME_GASOLINE_ROUTE_ID,
                    requirements: requirements
                }
            }
        }

        let register_msg: RegisterRequest = RegisterRequest{
            app_addr: ctx.info.sender.to_string(),
            route_criteria: vec![route_requirements]
        };

        let verifier_contract = self.verifier.load(ctx.deps.storage)?;

        // let register_msg = SubMsg::reply_on_success(
        //     register_msg.into_cosmos_msg(verifier_contract)?, 
        //     REGISTER_REQUIREMENT_REPLY_ID
        // );
        
        // Ok(Response::new().add_submessage(register_msg))

        Ok(Response::new()
            .add_message(register_msg.into_cosmos_msg(verifier_contract)?))
    }

    // Ask for the portion
    #[sv::msg(exec)]
    pub fn give_me_some_drink(&self, ctx: sylvia::types::ExecCtx, msg: GiveMeSomeDrink) -> StdResult<Response> {
        // 1. Save the transaction
        // 2. Send the request to verifier
        // 3. Wait for the reply
        PENDING_TRANSACTIONS.save(ctx.deps.storage, GIVE_ME_DRINK_ROUTE_ID, &ExecuteMsg::GiveMeSomeDrink(msg.clone()))?;
        // self.pending_transactions.save(ctx.deps.storage, &(ctx.info.clone(), ExecuteMsg::GiveMeSomeDrink(msg.clone())))?;
        let verifier_contract = self.verifier.load(ctx.deps.storage)?;
        
        let verify_request = SubMsg::reply_on_success(
            VerifyRequest{
                presentation: msg.proof,
                route_id: GIVE_ME_DRINK_ROUTE_ID,
                app_addr: Some(ctx.info.sender.to_string())
            }.into_cosmos_msg(verifier_contract)?,
            GIVE_ME_DRINK_ROUTE_ID,
        );
        

        Ok(Response::new()
            .add_submessage(verify_request)
        )
    }

    // Ask for the portion
    #[sv::msg(exec)]
    pub fn give_me_some_food(&self, ctx: sylvia::types::ExecCtx, msg: GiveMeSomeFood) -> StdResult<Response> {
        PENDING_TRANSACTIONS.save(ctx.deps.storage, GIVE_ME_FOOD_ROUTE_ID, &ExecuteMsg::GiveMeSomeFood(msg.clone()))?;
        let verifier_contract = self.verifier.load(ctx.deps.storage)?;
        let verify_request = SubMsg::reply_on_success( 
            VerifyRequest{
                presentation: msg.proof,
                route_id: GIVE_ME_FOOD_ROUTE_ID,
                app_addr: Some(ctx.info.sender.to_string())
            }.into_cosmos_msg(verifier_contract)?,
            GIVE_ME_FOOD_ROUTE_ID
        );

        Ok(Response::new()
            .add_submessage(verify_request)
        )

    }

    // Ask for the portion
    #[sv::msg(exec)]
    pub fn give_me_some_gasoline(&self, ctx: sylvia::types::ExecCtx, msg: GiveMeSomeGasoline) -> StdResult<Response> {
        PENDING_TRANSACTIONS.save(ctx.deps.storage, GIVE_ME_GASOLINE_ROUTE_ID, &ExecuteMsg::GiveMeSomeGasoline(msg.clone()))?;
        let verifier_contract = self.verifier.load(ctx.deps.storage)?;

        let verify_request = SubMsg::reply_on_success(
            VerifyRequest{
                presentation: msg.proof,
                route_id: GIVE_ME_GASOLINE_ROUTE_ID,
                app_addr: Some(ctx.info.sender.to_string())
            }.into_cosmos_msg(verifier_contract)?,
            GIVE_ME_GASOLINE_ROUTE_ID,
        );

        Ok(Response::new()
            .add_submessage(verify_request)
        )
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    // 
    match reply.id {
        REGISTER_REQUIREMENT_REPLY_ID => {
            let verification_result = parse_reply_execute_data(reply)?;
            if verification_result.data.is_none() {
                return Err(ContractError::VerificationProcessError)
            }
            return Ok(Response::new())
        }
        GIVE_ME_DRINK_ROUTE_ID => {
            let verification_result = parse_reply_execute_data(reply)?;
            let verified: bool = from_json(&verification_result.data.unwrap())?;
            let msg = PENDING_TRANSACTIONS.load(deps.storage, GIVE_ME_DRINK_ROUTE_ID)?;
            if verified {
                match msg {
                    ExecuteMsg::GiveMeSomeDrink(msg) => {
                        return Ok(Response::new()
                            .add_attribute("action", "give_me_some_drink")
                            .add_attribute("Drink kind", msg.kind)
                        )
                    }
                    _ => return Err(ContractError::VerificationProcessError)
                }
            }
            return Err(ContractError::VerificationProcessError)
        }
        GIVE_ME_FOOD_ROUTE_ID => {
            let verification_result = parse_reply_execute_data(reply)?;
            let verified: bool = from_json(&verification_result.data.unwrap())?;
            let msg = PENDING_TRANSACTIONS.load(deps.storage, GIVE_ME_FOOD_ROUTE_ID)?;
            if verified {
                match msg {
                    ExecuteMsg::GiveMeSomeFood(msg) => {
                        return Ok(Response::new()
                            .add_attribute("action", "give_me_some_food")
                            .add_attribute("Food kind", msg.kind)
                        )
                    }
                    _ => return Err(ContractError::VerificationProcessError)
                }
            }
            return Err(ContractError::VerificationProcessError)
        }
        GIVE_ME_GASOLINE_ROUTE_ID => {
            let verification_result = parse_reply_execute_data(reply)?;
            let verified: bool = from_json(&verification_result.data.unwrap())?;
            let msg = PENDING_TRANSACTIONS.load(deps.storage, GIVE_ME_GASOLINE_ROUTE_ID)?;
            if verified {
                match msg {
                    ExecuteMsg::GiveMeSomeGasoline(msg) => {
                        return Ok(Response::new()
                            .add_attribute("action", "give_me_some_gasoline")
                            .add_attribute("Amount of gasoline required", msg.amount)
                        )
                    }
                    _ => return Err(ContractError::VerificationProcessError)
                }
            }
            return Err(ContractError::VerificationProcessError)
        }
        _ => return Err(ContractError::InvalidRouteId)
    }
}
use avida_common::traits::avida_verifier_trait::sv::{AvidaVerifierTraitExecMsg, Querier};
use avida_common::types::{RegisterRouteRequest, RouteId, RouteVerificationRequirements};

use avida_sdjwt_verifier::{contract::SdjwtVerifier, types::VerifyResult};
use sylvia::cw_std::{from_json, Reply, Response, StdResult, SubMsg};
use sylvia::cw_std::{to_json_binary, WasmMsg};
use sylvia::types::{QueryCtx, Remote, ReplyCtx};
use sylvia::{contract, entry_points, schemars, types::InstantiateCtx};

use cw_storage_plus::{Item, Map};
use cw_utils::{parse_execute_response_data, MsgExecuteContractResponse};

use crate::constants::{
    GIVE_ME_DRINK_ROUTE_ID, GIVE_ME_FOOD_ROUTE_ID, REGISTER_REQUIREMENT_REPLY_ID,
};
use crate::error::ContractError;
use crate::types::{
    GetVerifierResponse, GiveMeSomeDrink, GiveMeSomeFood, OrderSubject, RegisterRequirement,
};

pub struct RestaurantContract<'a> {
    verifier: Item<'a, String>,
    pending_order_subjects: Map<'a, u64, OrderSubject>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
impl RestaurantContract<'_> {
    pub const fn new() -> Self {
        Self {
            verifier: Item::new("verifier"),
            pending_order_subjects: Map::new("pending_transactions"),
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
    pub fn register_requirement(
        &self,
        ctx: sylvia::types::ExecCtx,
        msg: RegisterRequirement,
    ) -> StdResult<Response> {
        let route_requirements: RegisterRouteRequest = match msg {
            RegisterRequirement::Drink { requirements } => RegisterRouteRequest {
                route_id: GIVE_ME_DRINK_ROUTE_ID,
                requirements,
            },
            RegisterRequirement::Food { requirements } => RegisterRouteRequest {
                route_id: GIVE_ME_FOOD_ROUTE_ID,
                requirements,
            },
        };

        let register_msg = AvidaVerifierTraitExecMsg::Register {
            app_addr: ctx.env.contract.address.to_string(),
            requests: vec![route_requirements],
        };

        let verifier_contract = self.verifier.load(ctx.deps.storage)?;

        let sub_msg = SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: verifier_contract,
                msg: to_json_binary(&register_msg)?,
                funds: vec![],
            },
            REGISTER_REQUIREMENT_REPLY_ID,
        );

        Ok(Response::new().add_submessage(sub_msg))
    }

    // Ask for the portion
    #[sv::msg(exec)]
    pub fn give_me_some_drink(
        &self,
        ctx: sylvia::types::ExecCtx,
        msg: GiveMeSomeDrink,
    ) -> StdResult<Response> {
        // 1. Save the transaction
        // 2. Send the request to verifier
        self.pending_order_subjects
            .save(ctx.deps.storage, GIVE_ME_DRINK_ROUTE_ID, &msg.kind)?;
        let verifier_contract = self.verifier.load(ctx.deps.storage)?;

        let verify_request = AvidaVerifierTraitExecMsg::Verify {
            presentation: msg.proof,
            route_id: GIVE_ME_DRINK_ROUTE_ID,
            app_addr: Some(ctx.env.contract.address.to_string()),
            additional_requirements: None,
        };
        let sub_msg = SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: verifier_contract,
                msg: to_json_binary(&verify_request)?,
                funds: vec![],
            },
            GIVE_ME_DRINK_ROUTE_ID,
        );

        Ok(Response::new().add_submessage(sub_msg))
    }

    // Ask for the portion
    #[sv::msg(exec)]
    pub fn give_me_some_food(
        &self,
        ctx: sylvia::types::ExecCtx,
        msg: GiveMeSomeFood,
    ) -> StdResult<Response> {
        // 1. Save the transaction
        // 2. Send the request to verifier
        self.pending_order_subjects
            .save(ctx.deps.storage, GIVE_ME_FOOD_ROUTE_ID, &msg.kind)?;
        let verifier_contract = self.verifier.load(ctx.deps.storage)?;
        let verify_request = AvidaVerifierTraitExecMsg::Verify {
            presentation: msg.proof,
            route_id: GIVE_ME_FOOD_ROUTE_ID,
            app_addr: Some(ctx.env.contract.address.to_string()),
            additional_requirements: None,
        };

        let sub_msg = SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: verifier_contract,
                msg: to_json_binary(&verify_request)?,
                funds: vec![],
            },
            GIVE_ME_FOOD_ROUTE_ID,
        );

        Ok(Response::new().add_submessage(sub_msg))
    }

    #[sv::msg(query)]
    fn get_verifier_address(&self, ctx: QueryCtx) -> Result<GetVerifierResponse, ContractError> {
        let verifier = self.verifier.load(ctx.deps.storage)?;
        Ok(GetVerifierResponse { verifier })
    }

    #[sv::msg(query)]
    fn get_route_requirements(
        &self,
        ctx: QueryCtx,
        route_id: RouteId,
    ) -> Result<RouteVerificationRequirements, ContractError> {
        let app_addr = ctx.env.contract.address.to_string();
        let verifier_addr = ctx
            .deps
            .api
            .addr_validate(self.verifier.load(ctx.deps.storage)?.as_str())?;
        let remote_contract = Remote::<SdjwtVerifier>::new(verifier_addr);
        let requirements = remote_contract
            .querier(&ctx.deps.querier)
            .get_route_requirements(app_addr, route_id)?;

        Ok(requirements)
    }

    #[sv::msg(reply)]
    fn reply(&self, ctx: ReplyCtx, reply: Reply) -> Result<Response, ContractError> {
        match (reply.id, reply.result.into_result()) {
            (REGISTER_REQUIREMENT_REPLY_ID, Err(err)) => Err(ContractError::RegistrationError(err)),
            (REGISTER_REQUIREMENT_REPLY_ID, Ok(_)) => Ok(Response::new()),
            (GIVE_ME_DRINK_ROUTE_ID | GIVE_ME_FOOD_ROUTE_ID, Err(err)) => {
                Err(ContractError::VerificationProcessError(err))
            }
            (rid @ GIVE_ME_DRINK_ROUTE_ID, Ok(res)) | (rid @ GIVE_ME_FOOD_ROUTE_ID, Ok(res)) => {
                if let MsgExecuteContractResponse {
                    data: Some(verify_result_bz),
                } = parse_execute_response_data(&res.data.ok_or(
                    ContractError::VerificationProcessError("VerifyResult not set".to_string()),
                )?)? {
                    let verify_result: VerifyResult = from_json(verify_result_bz)?;
                    match verify_result.result {
                        Ok(_) if rid == GIVE_ME_DRINK_ROUTE_ID => Ok(Response::new()
                            .add_attribute("action", "give_me_some_drink")
                            .add_attribute(
                                "Drink kind",
                                self.pending_order_subjects.load(ctx.deps.storage, rid)?,
                            )),
                        Ok(_) => Ok(Response::new()
                            .add_attribute("action", "give_me_some_food")
                            .add_attribute(
                                "Food kind",
                                self.pending_order_subjects.load(ctx.deps.storage, rid)?,
                            )),
                        Err(err) => Err(ContractError::VerificationProcessError(format!(
                            "{:?}",
                            err
                        ))),
                    }
                } else {
                    Err(ContractError::VerificationProcessError(
                        "VerifyResult not set".to_string(),
                    ))
                }
            }
            _ => Err(ContractError::InvalidRouteId),
        }
    }
}

impl Default for RestaurantContract<'_> {
    fn default() -> Self {
        Self::new()
    }
}

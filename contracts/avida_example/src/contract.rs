use avida_common::traits::avida_verifier_trait::sv::{AvidaVerifierTraitExecMsg, Querier};
use avida_common::types::{InputRoutesRequirements, RouteId, RouteVerificationRequirements};

use avida_sdjwt_verifier::{contract::SdjwtVerifier, types::VerifyResult};
use sylvia::cw_std::{from_json, Reply, Response, StdResult, SubMsg};
use sylvia::cw_std::{to_json_binary, WasmMsg};
use sylvia::types::{QueryCtx, Remote, ReplyCtx};
use sylvia::{contract, entry_points, schemars, types::InstantiateCtx};

use cw_storage_plus::{Item, Map};
use cw_utils::parse_reply_execute_data;

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
        let route_requirements: InputRoutesRequirements = match msg {
            RegisterRequirement::Drink { requirements } => InputRoutesRequirements {
                route_id: GIVE_ME_DRINK_ROUTE_ID,
                requirements,
            },
            RegisterRequirement::Food { requirements } => InputRoutesRequirements {
                route_id: GIVE_ME_FOOD_ROUTE_ID,
                requirements,
            },
        };

        let register_msg = AvidaVerifierTraitExecMsg::Register {
            app_addr: ctx.env.contract.address.to_string(),
            route_criteria: vec![route_requirements],
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
        let rid = reply.id;
        match rid {
            REGISTER_REQUIREMENT_REPLY_ID => match reply.result.into_result() {
                Err(err) => Err(ContractError::RegistrationError(err)),
                Ok(_) => Ok(Response::new()),
            },
            GIVE_ME_DRINK_ROUTE_ID | GIVE_ME_FOOD_ROUTE_ID => {
                match reply.clone().result.into_result() {
                    Err(err) => Err(ContractError::VerificationProcessError(err)),
                    Ok(_) => {
                        let verification_result = parse_reply_execute_data(reply)?;
                        match verification_result.data {
                            Some(data) => {
                                let verify_res: VerifyResult = from_json(data)?;
                                match verify_res.result {
                                    Ok(_) => {
                                        let order_subject = self
                                            .pending_order_subjects
                                            .load(ctx.deps.storage, rid)?;

                                        if rid == GIVE_ME_DRINK_ROUTE_ID {
                                            Ok(Response::new()
                                                .add_attribute("action", "give_me_some_drink")
                                                .add_attribute("Drink kind", order_subject))
                                        } else {
                                            Ok(Response::new()
                                                .add_attribute("action", "give_me_some_food")
                                                .add_attribute("Food kind", order_subject))
                                        }
                                    }
                                    Err(err) => {
                                        Err(ContractError::VerificationError(format!("{:?}", err)))
                                    }
                                }
                            }
                            None => Err(ContractError::VerificationProcessError(
                                "Data from reply cannot be empty".to_string(),
                            )),
                        }
                    }
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

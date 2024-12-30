use avida_common::types::RegisterRouteRequest;

use crate::msg::ExecuteMsg;
use avida_sdjwt_verifier::msg::ExecuteMsg as AvidaExecuteMsg;
use avida_sdjwt_verifier::types::VerifyResult;
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_json, to_json_binary, Binary, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw_utils::{parse_execute_response_data, MsgExecuteContractResponse};

use crate::constants::{
    GIVE_ME_DRINK_ROUTE_ID, GIVE_ME_FOOD_ROUTE_ID, REGISTER_REQUIREMENT_REPLY_ID,
};
use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{PENDING_ORDER_SUBJECTS, VERIFIER};
use crate::types::{GetVerifierResponse, GiveMeSomeDrink, GiveMeSomeFood, RegisterRequirement};

#[cfg_attr(not(feature = "library"), entry_point)]
fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
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
            )?)
            .map_err(|_| ContractError::ParseReplyError)?
            {
                let verify_result: VerifyResult = from_json(verify_result_bz)?;
                match verify_result.result {
                    Ok(_) if rid == GIVE_ME_DRINK_ROUTE_ID => Ok(Response::new()
                        .add_attribute("action", "give_me_some_drink")
                        .add_attribute(
                            "Drink kind",
                            PENDING_ORDER_SUBJECTS.load(deps.storage, rid)?,
                        )),
                    Ok(_) => Ok(Response::new()
                        .add_attribute("action", "give_me_some_food")
                        .add_attribute(
                            "Food kind",
                            PENDING_ORDER_SUBJECTS.load(deps.storage, rid)?,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.addr_validate(&msg.verifier)?;
    VERIFIER.save(deps.storage, &msg.verifier)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterRequirement { requirements } => {
            handle_register_requirement(deps, env, requirements)
        }
        ExecuteMsg::GiveMeSomeDrink(give_me_some_drink) => {
            handle_give_me_some_drink(deps, env, give_me_some_drink)
        }
        ExecuteMsg::GiveMeSomeFood(give_me_some_food) => {
            handle_give_me_some_food(deps, env, give_me_some_food)
        }
    }
}

// Register the permission policy
pub fn handle_register_requirement(
    deps: DepsMut,
    env: Env,
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

    let register_msg = AvidaExecuteMsg::Register {
        app_addr: env.contract.address.to_string(),
        requests: vec![route_requirements],
    };

    let verifier_contract = VERIFIER.load(deps.storage)?;

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
pub fn handle_give_me_some_drink(
    deps: DepsMut,
    env: Env,
    msg: GiveMeSomeDrink,
) -> StdResult<Response> {
    // 1. Save the transaction
    // 2. Send the request to verifier
    PENDING_ORDER_SUBJECTS.save(deps.storage, GIVE_ME_DRINK_ROUTE_ID, &msg.kind)?;
    let verifier_contract = VERIFIER.load(deps.storage)?;

    let verify_request = AvidaExecuteMsg::Verify {
        presentation: msg.proof,
        route_id: GIVE_ME_DRINK_ROUTE_ID,
        app_addr: Some(env.contract.address.to_string()),
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
pub fn handle_give_me_some_food(
    deps: DepsMut,
    env: Env,
    msg: GiveMeSomeFood,
) -> StdResult<Response> {
    // 1. Save the transaction
    // 2. Send the request to verifier
    PENDING_ORDER_SUBJECTS.save(deps.storage, GIVE_ME_FOOD_ROUTE_ID, &msg.kind)?;
    let verifier_contract = VERIFIER.load(deps.storage)?;
    let verify_request = AvidaExecuteMsg::Verify {
        presentation: msg.proof,
        route_id: GIVE_ME_FOOD_ROUTE_ID,
        app_addr: Some(env.contract.address.to_string()),
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

#[cfg_attr(not(feature = "library"), entry_point)]
fn query(deps: DepsMut, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVerifierAddress {} => to_json_binary(&get_verifier_address(deps)?),
    }
}

fn get_verifier_address(deps: DepsMut) -> Result<GetVerifierResponse, ContractError> {
    let verifier = VERIFIER.load(deps.storage)?;
    Ok(GetVerifierResponse { verifier })
}

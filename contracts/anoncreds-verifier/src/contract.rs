use crate::consts::{CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, Event, IbcChannelOpenMsg,
    IbcChannelOpenResponse, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use sylvia::{
    contract, schemars,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};

use ssi::traits::{
    resource_over_ibc_interface,
    resource_over_ibc_interface::{ResourceOverIbcError, ResourceOverIbcInterface},
};

pub struct AnonCredsVerifier {}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(ContractError)]
#[messages(resource_over_ibc_interface as ResourceOverIbcInterface)]
impl AnonCredsVerifier {
    pub const fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, ContractError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        let event = Event::new("vectis.AnonCredsVerifier.v1");
        Ok(Response::new().add_event(event))
    }

    fn ibc_channel_open(
        deps: DepsMut,
        env: Env,
        msg: IbcChannelOpenMsg,
    ) -> Result<IbcChannelOpenResponse, ContractError> {
        crate::ibc::ibc_channel_open(deps, env, msg)
    }
}

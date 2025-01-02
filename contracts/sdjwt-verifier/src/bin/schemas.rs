use cosmwasm_schema::write_api;

use avida_sdjwt_verifier::msg::{InstantiateMsg, QueryMsg};
use avida_common::types::AvidaVerifierExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: AvidaVerifierExecuteMsg,
        query: QueryMsg,
    }
}

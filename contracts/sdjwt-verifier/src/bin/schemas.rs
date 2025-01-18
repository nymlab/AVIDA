use cosmwasm_schema::write_api;

use avida_common::types::AvidaVerifierExecuteMsg;
use avida_sdjwt_verifier::msg::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: AvidaVerifierExecuteMsg,
        query: QueryMsg,
    }
}

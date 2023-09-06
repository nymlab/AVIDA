use cosmwasm_schema::write_api;

use vectis_anoncreds_verifier::contract::{ExecMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecMsg,
    }
}

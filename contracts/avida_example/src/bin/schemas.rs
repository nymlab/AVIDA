use cosmwasm_schema::write_api;

use avida_example::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}

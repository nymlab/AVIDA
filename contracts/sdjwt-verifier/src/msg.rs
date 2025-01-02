use avida_common::types::{RouteId, RouteVerificationRequirements};
use cosmwasm_schema::QueryResponses;

use crate::types::InitRegistration;

// Contract instantiation parameters
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub max_presentation_len: usize,
    pub init_registrations: Vec<InitRegistration>,
}

// Query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<String>)]
    GetRouteVerificationKey { app_addr: String, route_id: RouteId },
    #[returns(String)]
    GetAppAdmin { app_addr: String },
    #[returns(Vec<RouteId>)]
    GetRoutes { app_addr: String },
    #[returns(RouteVerificationRequirements)]
    GetRouteRequirements { app_addr: String, route_id: RouteId },
}

use avida_common::types::{
    RegisterRouteRequest, RouteId, RouteVerificationRequirements, VerfiablePresentation,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Binary;

use crate::types::{InitRegistration, UpdateRevocationListRequest};

// Contract instantiation parameters
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub max_presentation_len: usize,
    pub init_registrations: Vec<InitRegistration>,
}

// Execute messages
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    UpdateRevocationList {
        app_addr: String,
        request: UpdateRevocationListRequest,
    },
    Register {
        app_addr: String,
        requests: Vec<RegisterRouteRequest>,
    },
    Verify {
        presentation: VerfiablePresentation,
        route_id: RouteId,
        app_addr: Option<String>,
        additional_requirements: Option<Binary>,
    },
    Update {
        app_addr: String,
        route_id: RouteId,
        route_criteria: Option<RouteVerificationRequirements>,
    },
    Deregister {
        app_addr: String,
    },
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

// Sudo messages (privileged operations)
#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    Verify {
        app_addr: String,
        route_id: RouteId,
        presentation: VerfiablePresentation,
        additional_requirements: Option<Binary>,
    },
    Update {
        app_addr: String,
        route_id: RouteId,
        route_criteria: Option<RouteVerificationRequirements>,
    },
    Register {
        app_addr: String,
        app_admin: String,
        routes: Vec<RegisterRouteRequest>,
    },
}

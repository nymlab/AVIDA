// State structure

use std::collections::HashMap;

use avida_common::types::RouteId;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::types::{PendingRoute, VerificationRequirements};

pub const APP_ROUTES_REQUIREMENTS: Map<&str, HashMap<RouteId, VerificationRequirements>> =
    Map::new("app_routes_requirements");
pub const APP_ADMINS: Map<&str, Addr> = Map::new("app_admins");
pub const CHANNEL_ID: Item<String> = Item::new("channel_id");
pub const PENDING_VERIFICATION_REQ_REQUESTS: Map<&str, PendingRoute> =
    Map::new("pending_verification_req_requests");

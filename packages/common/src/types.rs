use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cw_storage_plus::Item;
/// The verifiable presentation type is encoded as Binary
pub type VerfiablePresentation = Binary;
pub const MAX_PRESENTATION_LENGTH: Item<usize> = Item::new("max_presentation_length");

pub type RouteId = u64;

/// Routes Requiments used in Registration (and Initiation)
#[cw_serde]
pub struct RegisterRouteRequest {
    pub route_id: RouteId,
    pub requirements: RouteVerificationRequirements,
}

/// A Sd-jwt specific requirement for revocation list update
/// using Criterion::NotContainedIn
#[cw_serde]
pub struct UpdateRevocationListRequest {
    pub route_id: u64,
    pub revoke: Vec<u64>,
    pub unrevoke: Vec<u64>,
}

/// Specific verification requirements for the route, by `route_id`
#[cw_serde]
pub struct RouteVerificationRequirements {
    /// This defines where the source data for verification is
    pub issuer_source_or_data: Vec<IssuerSourceOrData>,
    /// The presentation request is the criteria required for the presentation,
    /// for example required certains claims to be disclosed
    /// This value is stored as `VerificationRequirements.presentation_required` on sdjwtVerifier
    pub presentation_required: Option<Binary>,
}

#[cw_serde]
pub enum TrustRegistry {
    Cheqd = 1,
}

/// Location to obtain the verification data from
#[cw_serde]
pub struct IssuerSourceOrData {
    /// If `None`, this means data is directly provided
    pub source: Option<TrustRegistry>,
    /// The data or location of the verification data at the trust registry
    /// For TrustRegistry::Cheqd, it is the `ResourceReqPacket` in avida-cheqd
    /// For data, the contracts should have the expected type
    /// In Sdjwt-Verifier, this is expected to be jwk
    //
    pub data_or_location: Binary,
}

// Sudo messages (privileged operations)
#[cw_serde]
pub enum AvidaVerifierSudoMsg {
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

// Execute messages
#[cw_serde]
pub enum AvidaVerifierExecuteMsg {
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

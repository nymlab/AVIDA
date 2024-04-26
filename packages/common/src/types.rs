use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cw_storage_plus::{Item, Map};

/// This is set for the verifier to prevent the presentation from being too large
pub type MaxPresentationLen<'a> = Item<'a, usize>;
pub const MAX_PRESENTATION_LEN: MaxPresentationLen = Item::new("mpl");

/// The verifiable presentation type is encoded as Binary
pub type VerfiablePresentation = Binary;

/// For each route registered by a dApp smart contract,
/// the requirements are stored and updatable
pub type VerificationRequirements<'a> = Map<'a, RouteId, RouteVerificationRequirements>;

pub type RouteId = u64;

/// Specific verification requirements for the route, by `route_id`
#[cw_serde]
pub struct RouteVerificationRequirements {
    pub verification_source: VerificationSource,
    pub presentation_request: Binary,
}

#[cw_serde]
pub enum TrustRegistry {
    Cheqd = 1,
}

/// Location to obtain the verification data from
#[cw_serde]
pub struct VerificationSource {
    /// If `None`, this means data is directly provided
    pub source: Option<TrustRegistry>,
    /// The data or location of the verification data at the trust registry
    /// For TrustRegistry::Cheqd, it is the `ResourceReqPacket` in avida-cheqd
    /// For data, the contracts should have the expected type
    pub data_or_location: Binary,
}

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cw_storage_plus::{Item, Map};

/// This is set for the verifier to prevent the presentation from being too large
pub type MaxPresentationLen<'a> = Item<'a, usize>;
pub const MAX_PRESENTATION_LEN: MaxPresentationLen = Item::new("mpl");

/// The verifiable presentation type is encoded as Binary
pub type VerifiablePresentation = Binary;

/// For each route registered by a dApp smart contract,
/// the requirements are stored and updatable
pub type VerificationRequirements<'a> = Map<'a, RouteId, RouteVerificationRequirements>;

pub type RouteId = u64;

/// Routes Requiments used in Registration (and Initiation)
#[cw_serde]
pub struct InputRoutesRequirements {
    pub route_id: RouteId,
    pub requirements: RouteVerificationRequirements,
}

/// Specific verification requirements for the route, by `route_id`
#[cw_serde]
pub struct RouteVerificationRequirements {
    /// This defines where the source data for verification is
    pub verification_source: VerificationSource,
    /// The presentation request is the criteria required for the presentation,
    /// for example required certains claims to be disclosed
    /// This value is stored as `VerificationReq.presentation_required` on sdjwtVerifier
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
    /// In Sdjwt-Verifier, this is expected to be jwk
    pub data_or_location: Binary,
}

#[cw_serde]
pub enum AvidaVerifierSudoMsg {
    Verify {
        route_id: RouteId,
        presentation: Binary,
        app_addr: String,
    },
}

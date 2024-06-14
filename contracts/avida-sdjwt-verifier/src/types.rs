use super::errors::SdjwtVerifierError;

use avida_common::types::InputRoutesRequirements;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::from_json;
use cosmwasm_std::Binary;
use cosmwasm_std::SubMsg;
use jsonwebtoken::jwk::Jwk;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InitRegistration {
    pub app_addr: String,
    pub app_admin: String,
    pub routes: Vec<InputRoutesRequirements>,
}

#[cw_serde]
pub struct PendingRoute {
    pub route_id: u64,
    pub app_addr: String,
}

/// The verification request type, which consists of the verification request and the ibc message
pub struct VerificationRequest {
    pub verification_request: VerificationReq,
    pub ibc_msg: Option<SubMsg>,
}

impl VerificationRequest {
    /// Create a new verification request
    pub fn new(verification_request: VerificationReq, ibc_msg: Option<SubMsg>) -> Self {
        VerificationRequest {
            verification_request,
            ibc_msg,
        }
    }
}

/// Verification requirements are provided on registration on a route
/// The presentation request
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VerificationReq {
    /// This is the required presentation criteria,
    /// it is sent from presentation_request in the `RouteVerificationRequirements`
    pub presentation_required: base_sdjwt_verifier::types::PresentationReq,
    /// Usig this type as it is ser/deserializable
    pub issuer_pubkey: Option<Jwk>,
}

impl VerificationReq {
    pub fn new(
        presentation_request: Binary,
        issuer_pubkey: Option<Jwk>,
    ) -> Result<Self, SdjwtVerifierError> {
        Ok(VerificationReq {
            presentation_required: from_json(presentation_request)?,
            issuer_pubkey,
        })
    }
}

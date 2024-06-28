use super::errors::{SdjwtVerifierError, SdjwtVerifierResultError};

use avida_common::types::InputRoutesRequirements;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, Binary, BlockInfo, SubMsg};
use cw_utils::Expiration;
use jsonwebtoken::jwk::Jwk;
use serde::{Deserialize, Serialize};

pub const CW_EXPIRATION: &str = "cw_exp";

#[cw_serde]
pub struct VerifyResult {
    pub result: Result<(), SdjwtVerifierResultError>,
}

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

pub type PresentationReq = Vec<(CriterionKey, Criterion)>;

/// Verification requirements are provided on registration on a route
/// The presentation request
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VerificationReq {
    /// This is the required presentation criteria,
    /// it is sent from presentation_request in the `RouteVerificationRequirements`
    pub presentation_required: PresentationReq,
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

/// The json key for the disclosed claims
pub type CriterionKey = String;

#[cw_serde]
pub enum Criterion {
    String(String),
    Number(u64, MathsOperator),
    Boolean(bool),
    Expires(bool),
}

#[cw_serde]
pub enum MathsOperator {
    GreaterThan,
    LessThan,
    EqualTo,
}

/// Validate the verified claims against the presentation request
pub fn validate(
    presentation_request: PresentationReq,
    verified_claims: serde_json::Value,
    block_info: &BlockInfo,
) -> Result<(), SdjwtVerifierResultError> {
    match verified_claims {
        serde_json::Value::Null if presentation_request.is_empty() => Ok(()),
        serde_json::Value::Null => Err(SdjwtVerifierResultError::DisclosedClaimNotFound(
            "null_disclosures".to_string(),
        )),
        serde_json::Value::Object(claims) => {
            for (key, criterion) in presentation_request {
                match (&criterion, claims.get(&key)) {
                    (Criterion::Expires(true), Some(serde_json::Value::String(exp)))
                        if key == CW_EXPIRATION =>
                    {
                        let expiration: Expiration =
                            serde_json_wasm::from_str(exp).map_err(|_| {
                                SdjwtVerifierResultError::ExpirationStringInvalid(exp.clone())
                            })?;
                        if expiration.is_expired(block_info) {
                            return Err(SdjwtVerifierResultError::PresentationExpired(expiration));
                        }
                    }
                    // if `Criterion::Expires(true)` is requested, then
                    // - the key must be `CW_EXPIRATION`
                    // - the value must be a string
                    (Criterion::Expires(true), invalid_val) => {
                        return Err(SdjwtVerifierResultError::ExpirationKeyOrValueInvalid(
                            key.to_string(),
                            format!("{:?}", invalid_val),
                        ));
                    }
                    (Criterion::String(c_val), Some(serde_json::Value::String(p_val))) => {
                        if p_val != c_val {
                            return Err(SdjwtVerifierResultError::CriterionValueFailed(key));
                        }
                    }
                    (Criterion::Number(c_val, op), Some(serde_json::Value::Number(p_val))) => {
                        if let Some(num) = p_val.as_u64() {
                            match op {
                                MathsOperator::GreaterThan => {
                                    if &num <= c_val {
                                        return Err(
                                            SdjwtVerifierResultError::CriterionValueFailed(key),
                                        );
                                    }
                                }
                                MathsOperator::LessThan => {
                                    if &num >= c_val {
                                        return Err(
                                            SdjwtVerifierResultError::CriterionValueFailed(key),
                                        );
                                    }
                                }
                                MathsOperator::EqualTo => {
                                    if &num != c_val {
                                        return Err(
                                            SdjwtVerifierResultError::CriterionValueFailed(key),
                                        );
                                    }
                                }
                            }
                        } else {
                            return Err(SdjwtVerifierResultError::CriterionValueNumberInvalid);
                        }
                    }
                    (Criterion::Boolean(c_val), Some(serde_json::Value::Bool(bool_val))) => {
                        if bool_val != c_val {
                            return Err(SdjwtVerifierResultError::CriterionValueFailed(key));
                        }
                    }
                    _ => {
                        return Err(SdjwtVerifierResultError::DisclosedClaimNotFound(format!(
                            "Expects claim to be: {:?} for key: {}",
                            criterion, &key
                        )));
                    }
                }
            }
            // For when there are no dislosure required
            Ok(())
        }
        _ => Err(SdjwtVerifierResultError::VerifiedClaimsTypeUnexpected),
    }
}

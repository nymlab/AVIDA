use super::errors::{SdjwtVerifierError, SdjwtVerifierResultError};

use avida_common::types::RegisterRouteRequest;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, Binary, BlockInfo, SubMsg};
use cw_utils::Expiration;
use jsonwebtoken::jwk::Jwk;
use serde::{Deserialize, Serialize};

/// This is the key to be used in claims that specifies expiration using `cw_util::Expiration`
pub const CW_EXPIRATION: &str = "cw_exp";

/// This is the  key to be used in revocation_list in Criterion::NotContainedIn(revocation_list)
pub const IDX: &str = "idx";

#[cw_serde]
pub struct VerifyResult {
    pub result: Result<(), SdjwtVerifierResultError>,
}

#[cw_serde]
pub struct InitRegistration {
    pub app_addr: String,
    pub app_admin: String,
    pub routes: Vec<RegisterRouteRequest>,
}

#[cw_serde]
pub struct PendingRoute {
    pub route_id: u64,
    pub app_addr: String,
}

/// This is an internal struct that is used to organised the input RegisterRouteRequest to handle
/// 1. if issuer data is on trust registry, it will contain an ibc_msg
/// 2. if issuer data is directly provided, it will validate the jwk
pub(crate) struct _RegistrationRequest {
    pub verification_requirements: VerificationRequirements,
    pub ibc_msg: Option<SubMsg>,
}

impl _RegistrationRequest {
    /// Create a new verification request
    pub fn new(
        verification_requirements: VerificationRequirements,
        ibc_msg: Option<SubMsg>,
    ) -> Self {
        _RegistrationRequest {
            verification_requirements,
            ibc_msg,
        }
    }
}

pub type PresentationReq = Vec<(CriterionKey, Criterion)>;

/// Verification requirements are provided on registration on a route
/// The presentation request
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VerificationRequirements {
    /// This is the required presentation criteria,
    /// it is sent from presentation_request in the `RouteVerificationRequirements`
    pub presentation_required: PresentationReq,
    /// Usig this type as it is ser/deserializable
    pub issuer_pubkey: Option<Jwk>,
}

impl VerificationRequirements {
    pub fn new(
        presentation_request: Binary,
        issuer_pubkey: Option<Jwk>,
    ) -> Result<Self, SdjwtVerifierError> {
        Ok(VerificationRequirements {
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
    /// this can be used in any form,
    /// but it is designed to be used with key IDX as a revocationlist
    NotContainedIn(Vec<u64>),
}

#[cw_serde]
pub enum MathsOperator {
    GreaterThan,
    LessThan,
    EqualTo,
}

/// A Sd-jwt specific requirement for revocation list update
/// using Criterion::NotContainedIn
#[cw_serde]
pub struct UpdateRevocationListRequest {
    pub route_id: u64,
    pub revoke: Vec<u64>,
    pub unrevoke: Vec<u64>,
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
                    // - the key must be `CW_EXPIRATION`, this avoid clashing with `exp` / `iat`
                    // type claims
                    // - the value must be a string
                    (Criterion::Expires(true), invalid_val) => {
                        return Err(SdjwtVerifierResultError::ExpirationKeyOrValueInvalid(
                            key.to_string(),
                            format!("{:?}", invalid_val),
                        ));
                    }
                    (
                        Criterion::NotContainedIn(revocation_list),
                        Some(serde_json::Value::Number(idx)),
                    ) => {
                        let idx_u64 = idx
                            .as_u64()
                            .ok_or(SdjwtVerifierResultError::CriterionValueNumberInvalid)?;
                        if revocation_list.contains(&idx_u64) {
                            return Err(SdjwtVerifierResultError::IdxRevoked(idx_u64));
                        }
                    }
                    (Criterion::String(c_val), Some(serde_json::Value::String(p_val))) => {
                        if p_val != c_val {
                            return Err(SdjwtVerifierResultError::CriterionValueFailed(key));
                        }
                    }
                    (Criterion::Number(c_val, op), Some(serde_json::Value::Number(p_val))) => {
                        if let Some(num) = p_val.as_u64() {
                            match op {
                                MathsOperator::GreaterThan if &num <= c_val => {
                                    return Err(SdjwtVerifierResultError::CriterionValueFailed(
                                        key,
                                    ));
                                }

                                MathsOperator::LessThan if &num >= c_val => {
                                    return Err(SdjwtVerifierResultError::CriterionValueFailed(
                                        key,
                                    ));
                                }

                                MathsOperator::EqualTo if &num != c_val => {
                                    return Err(SdjwtVerifierResultError::CriterionValueFailed(
                                        key,
                                    ));
                                }
                                _ => {}
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

use super::errors::SdjwtVerifierError;

use avida_common::types::InputRoutesRequirements;
use cosmwasm_schema::cw_serde;
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

/// The json key for the disclosed claims
pub type CriterionKey = String;

#[cw_serde]
pub enum Criterion {
    String(String),
    Number(u64, MathsOperator),
    Boolean(bool),
}

#[cw_serde]
pub enum MathsOperator {
    GreaterThan,
    LessThan,
    EqualTo,
}

pub fn validate(
    presentation_request: PresentationReq,
    verified_claims: serde_json::Value,
) -> Result<bool, SdjwtVerifierError> {
    if let serde_json::Value::Object(claims) = verified_claims {
        for (key, criterion) in presentation_request {
            if let Some(value) = claims.get(&key) {
                match (value, criterion) {
                    // matches the presentation value `p_val` with the criterion value `c_val`
                    (serde_json::Value::String(p_val), Criterion::String(c_val)) => {
                        if p_val != &c_val {
                            return Ok(false);
                        }
                    }
                    (serde_json::Value::Number(p_val), Criterion::Number(c_val, op)) => {
                        if p_val.is_u64() {
                            let num = p_val.as_u64().unwrap();

                            match op {
                                MathsOperator::GreaterThan => {
                                    if num <= c_val {
                                        return Ok(false);
                                    };
                                }
                                MathsOperator::LessThan => {
                                    if num >= c_val {
                                        return Ok(false);
                                    };
                                }
                                MathsOperator::EqualTo => {
                                    if num != c_val {
                                        return Ok(false);
                                    };
                                }
                            }
                        } else {
                            return Err(SdjwtVerifierError::CriterionValueNumberInvalid);
                        }
                    }
                    (serde_json::Value::Bool(bool_val), Criterion::Boolean(c_val)) => {
                        if bool_val != &c_val {
                            return Ok(false);
                        }
                    }
                    _ => return Err(SdjwtVerifierError::CriterionValueTypeUnexpected),
                };
            } else {
                return Err(SdjwtVerifierError::DisclosedClaimNotFound(key.to_string()));
            }
        }
    } else {
        return Err(SdjwtVerifierError::VerifiedClaimsTypeUnexpected);
    };

    //When there are no criteria required
    Ok(true)
}

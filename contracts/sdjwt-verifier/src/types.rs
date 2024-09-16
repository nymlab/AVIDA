use super::errors::{SdjwtVerifierError, SdjwtVerifierResultError};

use avida_common::types::RegisterRouteRequest;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, Binary, BlockInfo, SubMsg};
use cw_utils::Expiration;
use jsonwebtoken::jwk::Jwk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn same_variant(a: &Criterion, b: &Criterion) -> bool {
    if std::mem::discriminant(a) != std::mem::discriminant(b) {
        return false;
    }
    // If both are `Number`, ensure the `MathsOperator` is the same
    match (a, b) {
        (Criterion::Number(_, op_self), Criterion::Number(_, op_other)) => op_self == op_other,
        _ => true, // For all other variants, we only care about the discriminant
    }
}

fn is_nested_dynamic_req(criterion: &Criterion) -> bool {
    if let Criterion::Dynamic(inner) = criterion {
        !matches!(**inner, Criterion::Dynamic(_))
    } else {
        false
    }
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

    /// validate
    pub fn validated_criterion(&self) -> Result<(), SdjwtVerifierError> {
        for (_, criterion) in self.presentation_required.iter() {
            if let Criterion::Dynamic(inner) = criterion {
                if is_nested_dynamic_req(inner) {
                    return Err(SdjwtVerifierError::DynamicRequirementsNested);
                }
            }
        }
        Ok(())
    }

    pub fn process_dynamic_requirements(
        &mut self,
        dyn_requirements: Option<Binary>,
    ) -> Result<(), SdjwtVerifierError> {
        // There is nothing to process if there are no input dynamic requirements
        // This does not mean that presentation will be validated,
        // it just means that we are not updating the requirements
        if dyn_requirements.is_none() {
            return Ok(());
        }
        let dyn_requirements: PresentationReq = from_json(dyn_requirements.unwrap())?;

        self.validated_criterion()?;

        // Create a HashMap of strings associated with Dynamic criteria in all_criteria
        let all_map: HashMap<&String, &Criterion> = self
            .presentation_required
            .iter()
            .filter_map(|(key, criterion)| {
                if let Criterion::Dynamic(inner) = criterion {
                    Some((key, inner.as_ref()))
                } else {
                    None
                }
            })
            .collect();

        // Create a HashMap from dyn_criteria
        let dyn_map: HashMap<&String, &Criterion> =
            dyn_requirements.iter().map(|(key, c)| (key, c)).collect();

        // Check if the HashMaps are equal
        if all_map.len() != dyn_map.len() {
            return Err(SdjwtVerifierError::DynamicRequirementsNotMatched(
                all_map.len(),
                dyn_map.len(),
            ));
        }

        // Check if they have the same variant
        let all_match = all_map.iter().all(|(key, criterion)| {
            dyn_map.get(key).map_or(false, |dyn_criterion| {
                same_variant(criterion, dyn_criterion)
            })
        });

        // we replace the all requirements variant content with the dynamic one
        if all_match {
            // Replace the Dynamic criteria in all_criteria
            for (key, criterion) in self.presentation_required.iter_mut() {
                if let Some(&dyn_criterion) = dyn_map.get(key) {
                    *criterion = dyn_criterion.clone();
                }
            }
            Ok(())
        } else {
            Err(SdjwtVerifierError::DynamicRequirementsVariantsMismatch)
        }
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
    // dynamic means that the actual expected will be replaced with a dynamic value provided when
    // verifying.
    // For example: a Criterion::Dynamic(Box<Criterion::Number(18, MathsOperator::GreaterThan)>)
    // will be replace by the the `dynamic_requirements_args`, for `Number` only the u64 is
    // replaced.
    Dynamic(Box<Criterion>),
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
                    (Criterion::Dynamic(_), _) => {
                        return Err(SdjwtVerifierResultError::DynamicRequirementNotProvided);
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

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::to_json_binary;

    #[test]
    fn process_dyn_req_nested_errors() {
        let mut nested_dyn = VerificationRequirements {
            presentation_required: vec![(
                "nested".to_string(),
                Criterion::Dynamic(Box::new(Criterion::Dynamic(Box::new(Criterion::Number(
                    18,
                    MathsOperator::GreaterThan,
                ))))),
            )],
            issuer_pubkey: None,
        };

        let dyn_req = Some(
            to_json_binary(&vec![(
                "nested".to_string(),
                Criterion::Number(20, MathsOperator::GreaterThan),
            )])
            .unwrap(),
        );

        let err = nested_dyn
            .process_dynamic_requirements(dyn_req)
            .unwrap_err();

        assert_eq!(err, SdjwtVerifierError::DynamicRequirementsNested);
    }

    #[test]
    fn process_dynamic_requirements_updates_dyn_correctly() {
        let mut all_req = VerificationRequirements {
            presentation_required: vec![
                (
                    "string_req".to_string(),
                    Criterion::String("John".to_string()),
                ),
                (
                    "number_req".to_string(),
                    Criterion::Number(20, MathsOperator::GreaterThan),
                ),
                ("bool_req".to_string(), Criterion::Boolean(true)),
                (
                    "dyn_req_number".to_string(),
                    Criterion::Dynamic(Box::new(Criterion::Number(0, MathsOperator::GreaterThan))),
                ),
            ],
            issuer_pubkey: None,
        };

        let new_criterion = Criterion::Number(20, MathsOperator::GreaterThan);

        let dyn_req = Some(
            to_json_binary(&vec![("dyn_req_number".to_string(), new_criterion.clone())]).unwrap(),
        );

        all_req.process_dynamic_requirements(dyn_req).unwrap();

        let new_req = all_req
            .presentation_required
            .iter()
            .find(|(key, _)| key == "dyn_req_number")
            .unwrap();

        assert_eq!(new_req.1, new_criterion);
    }

    #[test]
    fn process_dynamic_requirements_rejects_differnt_maths_ops() {
        let mut all_req = VerificationRequirements {
            presentation_required: vec![(
                "dyn_req_number".to_string(),
                Criterion::Dynamic(Box::new(Criterion::Number(0, MathsOperator::GreaterThan))),
            )],
            issuer_pubkey: None,
        };

        let new_criterion = Criterion::Number(20, MathsOperator::LessThan);

        let dyn_req = Some(
            to_json_binary(&vec![("dyn_req_number".to_string(), new_criterion.clone())]).unwrap(),
        );

        let err = all_req.process_dynamic_requirements(dyn_req).unwrap_err();

        assert_eq!(err, SdjwtVerifierError::DynamicRequirementsVariantsMismatch);
    }

    #[test]
    fn process_dynamic_requirements_cannot_overwrite_non_dyn_req() {
        let mut all_req = VerificationRequirements {
            presentation_required: vec![(
                "number_req".to_string(),
                Criterion::Number(0, MathsOperator::GreaterThan),
            )],
            issuer_pubkey: None,
        };

        let new_criterion = Criterion::Number(20, MathsOperator::GreaterThan);

        let dyn_req =
            Some(to_json_binary(&vec![("number_req".to_string(), new_criterion.clone())]).unwrap());

        let err = all_req.process_dynamic_requirements(dyn_req).unwrap_err();

        assert_eq!(err, SdjwtVerifierError::DynamicRequirementsNotMatched(0, 1));
    }

    #[test]
    fn validated_criterion_rejects_nested() {
        let nested_dyn = VerificationRequirements {
            presentation_required: vec![(
                "nested".to_string(),
                Criterion::Dynamic(Box::new(Criterion::Dynamic(Box::new(Criterion::Number(
                    18,
                    MathsOperator::GreaterThan,
                ))))),
            )],
            issuer_pubkey: None,
        };

        let err = nested_dyn.validated_criterion().unwrap_err();

        assert_eq!(err, SdjwtVerifierError::DynamicRequirementsNested);
    }
}

//  CosmWasm / Sylvia lib
use cosmwasm_std::{ensure, from_json, to_json_binary, Addr, Binary, Response, StdError};
use cw2::set_contract_version;
use cw_storage_plus::Item;
use sylvia::{
    contract, schemars,
    types::{ExecCtx, InstantiateCtx},
};

use crate::types::{Criterion, MathsOperator, PresentationReq};

use avida_common::types::VerifiablePresentation;

use jsonwebtoken::{jwk::Jwk, DecodingKey};
use sd_jwt_rs::{SDJWTSerializationFormat, SDJWTVerifier};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The `invoice factory` structure stored in state
pub struct BaseSdjwtVerifier<'a> {
    pub avida_sdjwt_verifier: Item<'a, Addr>,
}

use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum BaseSdjwtVerifierError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("sdjwt {0}")]
    SdJwt(String),
    #[error("Verified Claims should be an Object Map")]
    VerifiedClaimsTypeUnexpected,
    #[error("Criterion Value Type Unexpected")]
    CriterionValueTypeUnexpected,
    #[error("Criterion Value Number Unexpected")]
    CriterionValueNumberInvalid,
    #[error("No Disclosed Claims {0}")]
    DisclosedClaimNotFound(String),
    #[error("String Conversion {0}")]
    StringConversion(String),
    #[error("Jwk Conversion {0}")]
    JwkError(String),
    #[error("Required Claims Not Satisfied")]
    RequiredClaimsNotSatisfied,
    #[error("Unauthorised")]
    Unauthorised,
}

#[cfg_attr(not(feature = "library"), sylvia::entry_points)]
#[contract]
#[sv::error(BaseSdjwtVerifierError)]
impl BaseSdjwtVerifier<'_> {
    pub fn new() -> Self {
        Self {
            avida_sdjwt_verifier: Item::new("avida_sdjwt_verifier"),
        }
    }

    /// Instantiates sdjwt verifier
    #[sv::msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, BaseSdjwtVerifierError> {
        let InstantiateCtx { deps, info, .. } = ctx;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        self.avida_sdjwt_verifier.save(deps.storage, &info.sender)?;

        Ok(Response::default())
    }

    /// Execute Verification and return result in `set_data`
    #[sv::msg(exec)]
    fn verify(
        &self,
        ctx: ExecCtx,
        presentation: VerifiablePresentation,
        presentation_required: PresentationReq,
        issuer_pubkey: Binary,
    ) -> Result<Response, BaseSdjwtVerifierError> {
        let key = from_json::<Jwk>(&issuer_pubkey)?;
        let decoding_key = DecodingKey::from_jwk(&key)
            .map_err(|e| BaseSdjwtVerifierError::JwkError(e.to_string()))?;

        let ExecCtx { deps, info, .. } = ctx;
        let avida_addr = self.avida_sdjwt_verifier.load(deps.storage)?;
        ensure!(
            info.sender == avida_addr,
            BaseSdjwtVerifierError::Unauthorised
        );

        // We verify the presentation
        let verified_claims = SDJWTVerifier::new(
            String::from_utf8(presentation.to_vec())
                .map_err(|e| BaseSdjwtVerifierError::StringConversion(e.to_string()))?,
            Box::new(move |_, _| decoding_key.clone()),
            None, // This version does not support key binding
            None, // This version does not support key binding
            SDJWTSerializationFormat::Compact,
        )
        .map_err(|e| BaseSdjwtVerifierError::SdJwt(e.to_string()))?
        .verified_claims;

        // We validate the verified claims against the requirements
        if let Ok(r) = validate(presentation_required, verified_claims) {
            Ok(Response::default().set_data(to_json_binary(&r)?))
        } else {
            Err(BaseSdjwtVerifierError::RequiredClaimsNotSatisfied)
        }
    }
}

impl Default for BaseSdjwtVerifier<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate(
    presentation_request: PresentationReq,
    verified_claims: serde_json::Value,
) -> Result<bool, BaseSdjwtVerifierError> {
    if let serde_json::Value::Object(claims) = verified_claims {
        for (key, criterion) in presentation_request.0 {
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
                            return Err(BaseSdjwtVerifierError::CriterionValueNumberInvalid);
                        }
                    }
                    (serde_json::Value::Bool(bool_val), Criterion::Boolean(c_val)) => {
                        if bool_val != &c_val {
                            return Ok(false);
                        }
                    }
                    _ => return Err(BaseSdjwtVerifierError::CriterionValueTypeUnexpected),
                };
            } else {
                return Err(BaseSdjwtVerifierError::DisclosedClaimNotFound(key));
            }
        }
    } else {
        return Err(BaseSdjwtVerifierError::VerifiedClaimsTypeUnexpected);
    };

    //When there are no criteria required
    Ok(true)
}

use crate::{
    errors::SdjwtVerifierError,
    types::{validate, PresentationReq, VerificationReq},
};
use avida_common::{
    traits::{avida_verifier_trait, AvidaVerifierTrait},
    types::{
        AvidaVerifierSudoMsg, MaxPresentationLen, RouteId, RouteVerificationRequirements,
        TrustRegistry, VerfiablePresentation, VerificationSource, MAX_PRESENTATION_LEN,
    },
};

use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Addr, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Map;
use std::collections::HashMap;
use sylvia::{
    contract, entry_points, schemars,
    types::{ExecCtx, InstantiateCtx, QueryCtx, SudoCtx},
};

use jsonwebtoken::{
    jwk::{Jwk, KeyAlgorithm},
    DecodingKey,
};
use sd_jwt_rs::{utils::jwt_payload_decode, SDJWTSerializationFormat, SDJWTVerifier};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The `invoice factory` structure stored in state
pub struct SdjwtVerifier<'a> {
    /// Max Presentation Length
    pub max_presentation_len: MaxPresentationLen<'a>,
    /// Registered Smart Contract addrs and routes
    pub app_trust_data_source: Map<'a, &'a str, HashMap<RouteId, VerificationSource>>,
    pub app_routes_requirements: Map<'a, &'a str, HashMap<RouteId, VerificationReq>>,
    /// Registered Smart Contract addrs and their admins
    pub app_admins: Map<'a, &'a str, Addr>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::error(SdjwtVerifierError)]
#[sv::messages(avida_verifier_trait as AvidaVerifierTrait)]
impl SdjwtVerifier<'_> {
    pub fn new() -> Self {
        Self {
            max_presentation_len: MAX_PRESENTATION_LEN,
            app_trust_data_source: Map::new("data_sources"),
            app_routes_requirements: Map::new("routes_requirements"),
            app_admins: Map::new("admins"),
        }
    }

    /// Instantiates sdjwt verifier
    #[sv::msg(instantiate)]
    fn instantiate(
        &self,
        ctx: InstantiateCtx,
        max_presentation_len: usize,
        // Vec of app_addr to their routes and requirements
        init_registrations: Vec<(
            String, // Admin
            String, // App Addr
            Vec<(RouteId, RouteVerificationRequirements)>,
        )>,
    ) -> Result<Response, SdjwtVerifierError> {
        let InstantiateCtx { deps, env, .. } = ctx;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        self.max_presentation_len
            .save(deps.storage, &max_presentation_len)?;

        for app in init_registrations {
            let admin = deps.api.addr_validate(&app.0)?;
            let app_addr = deps.api.addr_validate(&app.1)?;
            self._register(deps.storage, &admin, app_addr.as_str(), app.2)?;
        }

        Ok(Response::default())
    }
}

mod verifier {

    use cosmwasm_std::Storage;

    use super::*;

    #[contract(module=crate::contract)]
    #[sv::messages(avida_verifier_trait as AvidaVerifierTrait)]
    impl AvidaVerifierTrait for SdjwtVerifier<'_> {
        type Error = SdjwtVerifierError;

        #[sv::msg(sudo)]
        fn sudo(&self, ctx: SudoCtx, msg: AvidaVerifierSudoMsg) -> Result<Response, Self::Error> {
            let SudoCtx { deps, env } = ctx;
            match msg {
                AvidaVerifierSudoMsg::Verify {
                    app_addr,
                    route_id,
                    presentation,
                } => {
                    // In `Sudo`, the app address may be the `moduleAccount`
                    // https://github.com/cosmos/cosmos-sdk/blob/b795646c9b2a5098e774f1726f8eac114ad79b13/x/auth/proto/cosmos/auth/v1beta1/auth.proto#L30
                    SdjwtVerifier::new()._verify(deps, presentation, route_id, &app_addr)
                }
            }
        }

        /// Application registration
        /// The caller will be the "admin" of the dApp to update requirements
        #[sv::msg(exec)]
        fn register(
            &self,
            ctx: ExecCtx,
            app_addr: String,
            route_criteria: Vec<(RouteId, RouteVerificationRequirements)>,
        ) -> Result<Response, Self::Error> {
            let ExecCtx { deps, env, info } = ctx;
            let app_addr = deps.api.addr_validate(&app_addr)?;
            self._register(
                deps.storage,
                &info.sender,
                app_addr.as_str(),
                route_criteria,
            )
        }

        /// Verifiable Presentation Verifier for dApp contracts
        #[sv::msg(exec)]
        fn verify(
            &self,
            ctx: ExecCtx,
            // Compact format serialised  sd-jwt
            presentation: VerfiablePresentation,
            route_id: RouteId,
            app_addr: Option<String>,
        ) -> Result<Response, Self::Error> {
            let ExecCtx { deps, info, .. } = ctx;
            let app_addr = app_addr.unwrap_or_else(|| info.sender.to_string());
            let app_addr = deps.api.addr_validate(&app_addr)?;

            self._verify(deps, presentation, route_id, app_addr.as_str())
        }

        // For dApp to update their criteria verification criteria
        #[sv::msg(exec)]
        fn update(
            &self,
            ctx: ExecCtx,
            app_addr: String,
            route_id: RouteId,
            route_criteria: Option<RouteVerificationRequirements>,
        ) -> Result<Response, Self::Error> {
            let ExecCtx { deps, env, info } = ctx;
            let app_addr = deps.api.addr_validate(&app_addr)?;

            let app_admin = self.app_admins.load(deps.storage, app_addr.as_str())?;
            if app_admin != info.sender {
                return Err(SdjwtVerifierError::Unauthorised);
            }
            unimplemented!()
        }

        //For dApp contracts to deregister
        #[sv::msg(exec)]
        fn deregister(&self, ctx: ExecCtx, app_addr: String) -> Result<Response, Self::Error> {
            unimplemented!()
        }

        // Query available routes for a dApp contract
        #[sv::msg(query)]
        fn get_routes(&self, ctx: QueryCtx, app_addr: String) -> Result<Vec<RouteId>, Self::Error> {
            unimplemented!()
        }

        // Query requirements of a route for a dApp contract
        #[sv::msg(query)]
        fn get_route_requirements(
            &self,
            ctx: QueryCtx,
            app_addr: String,
            route_id: RouteId,
        ) -> Result<RouteVerificationRequirements, Self::Error> {
            unimplemented!()
        }
    }

    impl SdjwtVerifier<'_> {
        pub fn _verify(
            &self,
            deps: DepsMut,
            presentation: VerfiablePresentation,
            route_id: RouteId,
            app_addr: &str,
        ) -> Result<Response, SdjwtVerifierError> {
            // If app is registered, load the requirementes for the given route_id
            let requirements = self
                .app_routes_requirements
                .load(deps.storage, app_addr)?
                .get(&route_id)
                .ok_or(SdjwtVerifierError::RouteNotRegistered)?
                .clone();

            let decoding_key = DecodingKey::from_jwk(
                requirements
                    .issuer_pubkey
                    .as_ref()
                    .ok_or(SdjwtVerifierError::PubKeyNotFound)?,
            )
            .map_err(|e| SdjwtVerifierError::JwtError(e.to_string()))?;

            // We verify the presentation
            let verified_claims = SDJWTVerifier::new(
                String::from_utf8(presentation.to_vec())
                    .map_err(|e| SdjwtVerifierError::StringConversion(e.to_string()))?,
                Box::new(move |_, _| decoding_key.clone()),
                None, // This version does not support key binding
                None, // This version does not support key binding
                SDJWTSerializationFormat::Compact,
            )
            .map_err(|e| SdjwtVerifierError::SdJwt(e.to_string()))?
            .verified_claims;

            // We validate the verified claims against the requirements
            if let Ok(r) = validate(requirements.presentation_required, verified_claims) {
                Ok(Response::default().set_data(to_json_binary(&r)?))
            } else {
                Err(SdjwtVerifierError::RequiredClaimsNotSatisfied)
            }
        }

        pub fn _register(
            &self,
            storage: &mut dyn Storage,
            admin: &Addr,
            app_addr: &str,
            route_criteria: Vec<(RouteId, RouteVerificationRequirements)>,
        ) -> Result<Response, SdjwtVerifierError> {
            if self.app_trust_data_source.has(storage, app_addr)
                || self.app_routes_requirements.has(storage, app_addr)
            {
                return Err(SdjwtVerifierError::AppAlreadyRegistered);
            }

            let mut requirements: HashMap<u64, VerificationReq> = HashMap::new();
            let mut data_sources: HashMap<u64, VerificationSource> = HashMap::new();

            for (route_id, route_criteria) in route_criteria {
                data_sources.insert(route_id, route_criteria.verification_source.clone());
                // On registration we check if the dApp has request for IBC data
                // FIXME: add IBC submessages
                let verif_req = match route_criteria.verification_source.source {
                    Some(registry) => {
                        match registry {
                            TrustRegistry::Cheqd => {
                                // For Cheqd, the data is in the ResourceReqPacket
                                VerificationReq {
                                    presentation_required: from_json(
                                        route_criteria.presentation_request,
                                    )?,
                                    issuer_pubkey: None,
                                }
                            }
                        }
                    }
                    None => {
                        let issuer_pubkey: Jwk = serde_json_wasm::from_slice(
                            &route_criteria.verification_source.data_or_location,
                        )?;

                        if let Some(KeyAlgorithm::EdDSA) = issuer_pubkey.common.key_algorithm {
                            VerificationReq {
                                presentation_required: from_json(
                                    route_criteria.presentation_request,
                                )?,
                                issuer_pubkey: Some(issuer_pubkey),
                            }
                        } else {
                            return Err(SdjwtVerifierError::UnsupportedKeyType);
                        }
                    }
                };
                requirements.insert(route_id, verif_req);
            }

            self.app_trust_data_source
                .save(storage, app_addr, &data_sources)?;
            self.app_routes_requirements
                .save(storage, app_addr, &requirements)?;
            self.app_admins.save(storage, app_addr, admin)?;

            Ok(Response::default())
        }
    }
}

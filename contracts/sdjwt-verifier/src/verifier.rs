use avida_cheqd::types::ResourceReqPacket;
use cosmwasm_std::Storage;

use std::collections::HashMap;

pub use crate::{
    contract::SdjwtVerifier,
    errors::SdjwtVerifierError,
    types::{validate, PendingRoute, VerificationReq, VerificationRequest},
};

// AVIDA specific
use avida_cheqd::ibc::{get_timeout_timestamp, HOUR_PACKET_LIFETIME};
use avida_common::{
    traits::AvidaVerifierTrait,
    types::{
        AvidaVerifierSudoMsg, InputRoutesRequirements, RouteId, RouteVerificationRequirements,
        TrustRegistry, VerfiablePresentation, VerificationSource,
    },
};

//  CosmWasm / Sylvia lib
use cosmwasm_std::{
    from_json, to_json_binary, Addr, CosmosMsg, DepsMut, Env, IbcTimeout, Response, SubMsg,
};

use sylvia::{
    contract,
    types::{ExecCtx, QueryCtx, SudoCtx},
};

// sd-jwt specific dependencies
pub use jsonwebtoken::{
    jwk::{AlgorithmParameters, EllipticCurve, Jwk, OctetKeyPairParameters},
    DecodingKey,
};

use sd_jwt_rs::{SDJWTSerializationFormat, SDJWTVerifier};

#[contract(module=crate::contract)]
#[sv::messages(avida_verifier_trait as AvidaVerifierTrait)]
impl AvidaVerifierTrait for SdjwtVerifier<'_> {
    type Error = SdjwtVerifierError;

    #[sv::msg(sudo)]
    fn sudo(&self, ctx: SudoCtx, msg: AvidaVerifierSudoMsg) -> Result<Response, Self::Error> {
        let SudoCtx { deps, env: _ } = ctx;
        match msg {
            AvidaVerifierSudoMsg::Verify {
                app_addr,
                route_id,
                presentation,
            } => {
                // In `Sudo`, the app address may be the `moduleAccount`
                self._verify(deps, presentation, route_id, &app_addr)
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
        route_criteria: Vec<InputRoutesRequirements>,
    ) -> Result<Response, Self::Error> {
        let ExecCtx { deps, env, info } = ctx;
        let app_addr = deps.api.addr_validate(&app_addr)?;

        // Complete registration
        self._register(
            deps.storage,
            &env,
            &info.sender,
            app_addr.as_str(),
            route_criteria,
        )
    }

    /// Performs the verification of the provided presentation within the context of the given route
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

        // Performs the verification of the provided presentation within the context of the given route
        self._verify(deps, presentation, route_id, app_addr.as_str())
    }

    /// For dApp to update their verification criteria
    #[sv::msg(exec)]
    fn update(
        &self,
        ctx: ExecCtx,
        app_addr: String,
        route_id: RouteId,
        route_criteria: Option<RouteVerificationRequirements>,
    ) -> Result<Response, Self::Error> {
        let ExecCtx { deps, env, info } = ctx;

        // Ensure the app with this address is registered
        if !self.app_trust_data_source.has(deps.storage, &app_addr)
            || !self.app_routes_requirements.has(deps.storage, &app_addr)
        {
            return Err(SdjwtVerifierError::AppIsNotRegistered);
        }

        let app_addr = deps.api.addr_validate(&app_addr)?;

        let app_admin = self.app_admins.load(deps.storage, app_addr.as_str())?;

        // Ensure the caller is the admin of the dApp
        if app_admin != info.sender {
            return Err(SdjwtVerifierError::Unauthorised);
        }

        // Perform verification criteria update
        self._update(
            deps.storage,
            &env,
            app_addr.as_str(),
            route_id,
            route_criteria,
        )
    }

    /// For dApp contracts to deregister
    #[sv::msg(exec)]
    fn deregister(&self, ctx: ExecCtx, app_addr: String) -> Result<Response, Self::Error> {
        let ExecCtx { deps, info, .. } = ctx;

        // Ensure the app with this address is registered
        if !self.app_trust_data_source.has(deps.storage, &app_addr)
            || !self.app_routes_requirements.has(deps.storage, &app_addr)
        {
            return Err(SdjwtVerifierError::AppIsNotRegistered);
        }

        let app_addr = deps.api.addr_validate(&app_addr)?;
        let app_admin = self.app_admins.load(deps.storage, app_addr.as_str())?;

        // Ensure the caller is the admin of the dApp
        if app_admin != info.sender {
            return Err(SdjwtVerifierError::Unauthorised);
        }

        // Perform deregistration
        self._deregister(deps.storage, app_addr.as_str())
    }

    /// Query available routes for a dApp contract
    #[sv::msg(query)]
    fn get_routes(&self, ctx: QueryCtx, app_addr: String) -> Result<Vec<RouteId>, Self::Error> {
        let v = self
            .app_routes_requirements
            .load(ctx.deps.storage, &app_addr)?;
        let routes: Vec<RouteId> = v.keys().cloned().collect();
        Ok(routes)
    }

    /// Query requirements of a route for a dApp contract
    #[sv::msg(query)]
    fn get_route_requirements(
        &self,
        ctx: QueryCtx,
        app_addr: String,
        route_id: RouteId,
    ) -> Result<RouteVerificationRequirements, Self::Error> {
        let req = self
            .app_routes_requirements
            .load(ctx.deps.storage, &app_addr)?;
        let route_req = req
            .get(&route_id)
            .ok_or(SdjwtVerifierError::RouteNotRegistered)?;

        let trust_data = self
            .app_trust_data_source
            .load(ctx.deps.storage, &app_addr)?;
        let route_td = trust_data
            .get(&route_id)
            .ok_or(SdjwtVerifierError::RouteNotRegistered)?;

        Ok(RouteVerificationRequirements {
            verification_source: route_td.clone(),
            presentation_request: to_json_binary(&route_req.presentation_required)?,
        })
    }
}

impl SdjwtVerifier<'_> {
    /// Verify the provided presentation within the context of the given route
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

    /// Performs a registration of an application and all its routes
    pub fn _register(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        admin: &Addr,
        app_addr: &str,
        route_criteria: Vec<InputRoutesRequirements>,
    ) -> Result<Response, SdjwtVerifierError> {
        if self.app_trust_data_source.has(storage, app_addr)
            || self.app_routes_requirements.has(storage, app_addr)
        {
            return Err(SdjwtVerifierError::AppAlreadyRegistered);
        }

        let mut req_map: HashMap<u64, VerificationReq> = HashMap::new();
        let mut data_sources: HashMap<u64, VerificationSource> = HashMap::new();

        let mut response = Response::default();

        for InputRoutesRequirements {
            route_id,
            requirements,
        } in route_criteria
        {
            data_sources.insert(route_id, requirements.verification_source.clone());
            // On registration we check if the dApp has request for IBC data
            // FIXME: add IBC submessages

            // Make a verification request for specified app addr and route id with a provided route criteria
            let VerificationRequest {
                verification_request,
                ibc_msg,
            } = self.make_verification_request(storage, env, app_addr, route_id, requirements)?;

            req_map.insert(route_id, verification_request);

            if let Some(ibc_msg) = ibc_msg {
                response = response.add_submessage(ibc_msg);
            }
        }

        // Save the registered trust data sources and route requirements
        self.app_trust_data_source
            .save(storage, app_addr, &data_sources)?;
        self.app_routes_requirements
            .save(storage, app_addr, &req_map)?;
        self.app_admins.save(storage, app_addr, admin)?;

        Ok(response)
    }

    /// Performs a deregister of an application and all its routes
    fn _deregister(
        &self,
        storage: &mut dyn Storage,
        app_addr: &str,
    ) -> Result<Response, SdjwtVerifierError> {
        self.app_trust_data_source.remove(storage, app_addr);
        self.app_routes_requirements.remove(storage, app_addr);
        self.app_admins.remove(storage, app_addr);

        Ok(Response::default())
    }

    /// Performs an update on the verification requirements for a given app addr and route id with the new criteria
    fn _update(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        app_addr: &str,
        route_id: RouteId,
        route_criteria: Option<RouteVerificationRequirements>,
    ) -> Result<Response, SdjwtVerifierError> {
        let mut req_map = self.app_routes_requirements.load(storage, app_addr)?;
        let mut data_sources = self.app_trust_data_source.load(storage, app_addr)?;

        let mut response: Response = Response::default();

        // On registration we check if the dApp has request for IBC data
        // FIXME: add IBC submessages
        if let Some(route_criteria) = route_criteria {
            data_sources.insert(route_id, route_criteria.verification_source.clone());

            // Make a verification request for specified app addr and route id with a provided route criteria
            let VerificationRequest {
                verification_request,
                ibc_msg,
            } = self.make_verification_request(storage, env, app_addr, route_id, route_criteria)?;

            req_map.insert(route_id, verification_request);

            if let Some(ibc_msg) = ibc_msg {
                response = response.add_submessage(ibc_msg);
            }
        } else {
            data_sources.remove(&route_id);
            req_map.remove(&route_id);
        }

        // Save the updated trust data sources and route requirements
        self.app_trust_data_source
            .save(storage, app_addr, &data_sources)?;
        self.app_routes_requirements
            .save(storage, app_addr, &req_map)?;

        Ok(response)
    }

    /// Creates a verification request for specified app addr and route id and provided route criteria
    fn make_verification_request(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        app_addr: &str,
        route_id: RouteId,
        route_criteria: RouteVerificationRequirements,
    ) -> Result<VerificationRequest, SdjwtVerifierError> {
        if let Some(registry) = route_criteria.verification_source.source {
            match registry {
                TrustRegistry::Cheqd => {
                    // For Cheqd, the data is in the ResourceReqPacket
                    let resource_req_packat: ResourceReqPacket =
                        from_json(&route_criteria.verification_source.data_or_location)?;

                    let ibc_msg = SubMsg::new(CosmosMsg::Ibc(cosmwasm_std::IbcMsg::SendPacket {
                        channel_id: self.channel_id.load(storage)?,
                        data: to_json_binary(&resource_req_packat)?,
                        timeout: IbcTimeout::with_timestamp(get_timeout_timestamp(
                            env,
                            HOUR_PACKET_LIFETIME,
                        )),
                    }));

                    self.pending_verification_req_requests.save(
                        storage,
                        &resource_req_packat.to_string(),
                        &PendingRoute {
                            app_addr: app_addr.to_string(),
                            route_id,
                        },
                    )?;

                    let verification_req: VerificationReq =
                        VerificationReq::new(route_criteria.presentation_request, None)?;
                    Ok(VerificationRequest::new(verification_req, Some(ibc_msg)))
                }
            }
        } else {
            let issuer_pubkey: Jwk =
                serde_json_wasm::from_slice(&route_criteria.verification_source.data_or_location)?;

            println!("issuer_pubkey: {:?}", issuer_pubkey);

            if let AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                curve: EllipticCurve::Ed25519,
                ..
            }) = issuer_pubkey.algorithm
            {
                let verification_req: VerificationReq =
                    VerificationReq::new(route_criteria.presentation_request, Some(issuer_pubkey))?;

                Ok(VerificationRequest::new(verification_req, None))
            } else {
                Err(SdjwtVerifierError::UnsupportedKeyType)
            }
        }
    }
}

use std::collections::HashMap;

use crate::{
    contract::SdjwtVerifier,
    errors::{SdjwtVerifierError, SdjwtVerifierResultError},
    types::{
        validate, PendingRoute, PresentationReq, VerificationRequirements, VerifyResult,
        _RegistrationRequest,
    },
};

// AVIDA specific
use avida_cheqd::{
    ibc::{get_timeout_timestamp, HOUR_PACKET_LIFETIME},
    types::ResourceReqPacket,
};
use avida_common::{
    traits::AvidaVerifierTrait,
    types::{
        AvidaVerifierSudoMsg, IssuerSourceOrData, RegisterRouteRequest, RouteId,
        RouteVerificationRequirements, TrustRegistry, VerfiablePresentation,
    },
};

//  CosmWasm / Sylvia lib
use cosmwasm_std::{
    ensure, from_json, to_json_binary, Addr, Binary, BlockInfo, CosmosMsg, Env, IbcTimeout,
    Response, Storage, SubMsg,
};

use sylvia::{
    contract,
    types::{ExecCtx, QueryCtx, SudoCtx},
};

// sd-jwt specific dependencies
use jsonwebtoken::{
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
        let SudoCtx { deps, env } = ctx;
        match msg {
            AvidaVerifierSudoMsg::Verify {
                app_addr,
                route_id,
                presentation,
                additional_requirements,
            } => {
                let additional_requirements: Option<PresentationReq> =
                    additional_requirements.map(from_json).transpose()?;
                // If app is registered, load the requirementes for the given route_id
                let requirements = self
                    .app_routes_requirements
                    .load(deps.storage, app_addr.as_str())?
                    .get(&route_id)
                    .ok_or(SdjwtVerifierError::RouteNotRegistered)?
                    .clone();
                let max_len = self.max_presentation_len.load(deps.storage)?;

                // In `Sudo`, the app address may be the `moduleAccount`
                Ok(self
                    ._verify(
                        presentation,
                        requirements,
                        max_len,
                        &env.block,
                        additional_requirements,
                    )
                    .map(|_| Response::default())
                    .map_err(SdjwtVerifierError::SdjwtVerifierResultError)?)
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
        requests: Vec<RegisterRouteRequest>,
    ) -> Result<Response, Self::Error> {
        let ExecCtx { deps, env, info } = ctx;
        let app_addr = deps.api.addr_validate(&app_addr)?;

        deps.api.debug(&format!("{:?}", requests));

        // Complete registration
        self._register(
            deps.storage,
            &env,
            &info.sender,
            app_addr.as_str(),
            requests,
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
        additional_requirements: Option<Binary>,
    ) -> Result<Response, Self::Error> {
        let ExecCtx { deps, info, env } = ctx;

        let additional_requirements: Option<PresentationReq> =
            additional_requirements.map(from_json).transpose()?;
        let app_addr = app_addr.unwrap_or_else(|| info.sender.to_string());
        let app_addr = deps.api.addr_validate(&app_addr)?;

        // If app is registered, load the requirementes for the given route_id
        let requirements = self
            .app_routes_requirements
            .load(deps.storage, app_addr.as_str())?
            .get(&route_id)
            .ok_or(SdjwtVerifierError::RouteNotRegistered)?
            .clone();
        let max_len = self.max_presentation_len.load(deps.storage)?;
        // Performs the verification of the provided presentation within the context of the given route
        let res = self._verify(
            presentation,
            requirements,
            max_len,
            &env.block,
            additional_requirements,
        );

        //:we response with the error so that it can be propagated
        Ok(Response::default().set_data(to_json_binary(&VerifyResult { result: res })?))
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
            issuer_source_or_data: route_td.clone(),
            presentation_required: to_json_binary(&route_req.presentation_required)?,
        })
    }
}

impl SdjwtVerifier<'_> {
    /// Verify the provided presentation within the context of the given route
    pub fn _verify(
        &self,
        presentation: VerfiablePresentation,
        requirements: VerificationRequirements,
        max_presentation_len: usize,
        block_info: &BlockInfo,
        additional_requirements: Option<PresentationReq>,
    ) -> Result<(), SdjwtVerifierResultError> {
        // Ensure the presentation is not too large
        ensure!(
            presentation.len() <= max_presentation_len,
            SdjwtVerifierResultError::PresentationTooLarge
        );

        let decoding_key = DecodingKey::from_jwk(
            requirements
                .issuer_pubkey
                .as_ref()
                .ok_or(SdjwtVerifierResultError::PubKeyNotFound)?,
        )
        .map_err(|e| SdjwtVerifierResultError::JwtError(e.to_string()))?;

        // We verify the presentation
        let verified_claims = SDJWTVerifier::new(
            String::from_utf8(presentation.to_vec())
                .map_err(|e| SdjwtVerifierResultError::StringConversion(e.to_string()))?,
            Box::new(move |_, _| decoding_key.clone()),
            None, // This version does not support key binding
            None, // This version does not support key binding
            SDJWTSerializationFormat::Compact,
        )
        .map_err(|e| SdjwtVerifierResultError::SdJwt(e.to_string()))?
        .verified_claims;

        let combined_requirements = if let Some(additional_requirements) = additional_requirements {
            let mut combined_requirements = requirements.presentation_required.clone();
            combined_requirements.extend(additional_requirements);
            combined_requirements
        } else {
            requirements.presentation_required.clone()
        };

        // We validate the verified claims against the requirements
        validate(combined_requirements, verified_claims, block_info)
    }

    /// Performs a registration of an application and all its routes
    pub fn _register(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        admin: &Addr,
        app_addr: &str,
        route_criteria: Vec<RegisterRouteRequest>,
    ) -> Result<Response, SdjwtVerifierError> {
        if self.app_trust_data_source.has(storage, app_addr)
            || self.app_routes_requirements.has(storage, app_addr)
        {
            return Err(SdjwtVerifierError::AppAlreadyRegistered);
        }

        let mut req_map: HashMap<u64, VerificationRequirements> = HashMap::new();
        let mut data_sources: HashMap<u64, IssuerSourceOrData> = HashMap::new();

        let mut response = Response::default();

        for RegisterRouteRequest {
            route_id,
            requirements,
        } in route_criteria
        {
            data_sources.insert(route_id, requirements.issuer_source_or_data.clone());
            // On registration we check if the dApp has request for IBC data
            // Make a verification request for specified app addr and route id with a provided route criteria
            let _RegistrationRequest {
                verification_requirements,
                ibc_msg,
            } = self.make_internal_registration_request(
                storage,
                env,
                app_addr,
                route_id,
                requirements,
            )?;

            req_map.insert(route_id, verification_requirements);

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
        if let Some(route_criteria) = route_criteria {
            data_sources.insert(route_id, route_criteria.issuer_source_or_data.clone());

            // Make a verification request for specified app addr and route id with a provided route criteria
            let _RegistrationRequest {
                verification_requirements,
                ibc_msg,
            } = self.make_internal_registration_request(
                storage,
                env,
                app_addr,
                route_id,
                route_criteria,
            )?;

            req_map.insert(route_id, verification_requirements);

            if let Some(ibc_msg) = ibc_msg {
                response = response.add_submessage(ibc_msg);
            }
        } else {
            data_sources.remove(&route_id);
            req_map.remove(&route_id);
        }

        if data_sources.is_empty() && req_map.is_empty() {
            // If there are no more routes, deregister the app
            self._deregister(storage, app_addr)
        } else {
            // Save the updated trust data sources and route requirements
            self.app_trust_data_source
                .save(storage, app_addr, &data_sources)?;
            self.app_routes_requirements
                .save(storage, app_addr, &req_map)?;

            Ok(response)
        }
    }

    /// Creates a _RegitrationRequest for specified app addr and route id and provided route criteria
    fn make_internal_registration_request(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        app_addr: &str,
        route_id: RouteId,
        route_criteria: RouteVerificationRequirements,
    ) -> Result<_RegistrationRequest, SdjwtVerifierError> {
        if let Some(registry) = route_criteria.issuer_source_or_data.source {
            match registry {
                TrustRegistry::Cheqd => {
                    // For Cheqd, the data is in the ResourceReqPacket
                    let resource_req_packat: ResourceReqPacket =
                        from_json(&route_criteria.issuer_source_or_data.data_or_location)?;

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

                    let verification_req =
                        VerificationRequirements::new(route_criteria.presentation_required, None)?;
                    Ok(_RegistrationRequest::new(verification_req, Some(ibc_msg)))
                }
            }
        } else {
            let issuer_pubkey: Jwk =
                from_json(&route_criteria.issuer_source_or_data.data_or_location)?;

            if let AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                curve: EllipticCurve::Ed25519,
                ..
            }) = issuer_pubkey.algorithm
            {
                let verification_req = VerificationRequirements::new(
                    route_criteria.presentation_required,
                    Some(issuer_pubkey),
                )?;

                Ok(_RegistrationRequest::new(verification_req, None))
            } else {
                Err(SdjwtVerifierError::UnsupportedKeyType)
            }
        }
    }
}

use crate::types::{
    AvidaVerifierSudoMsg, RegisterRouteRequest, RouteId, RouteVerificationRequirements,
    VerfiablePresentation,
};
use cosmwasm_std::{Binary, Response, StdError};
use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
use sylvia::{interface, schemars};

pub use avida_verifier_trait::AvidaVerifierTrait;

pub mod avida_verifier_trait {
    use super::*;

    /// The trait common for verifier contracts
    #[interface]
    pub trait AvidaVerifierTrait {
        type Error: From<StdError>;

        /// Application registration
        /// The caller will be the "admin" of the dApp to update requirements
        #[sv::msg(exec)]
        fn register(
            &self,
            ctx: ExecCtx,
            app_addr: String,
            requests: Vec<RegisterRouteRequest>,
        ) -> Result<Response, Self::Error>;

        /// Verifiable Presentation Verifier for dApp contracts
        /// additional_requirements is the dynamic added (per tx) requirements that can be passed to the verifier at the
        /// time of verification, for sdjwt, it is requirement for claims kv pair
        #[sv::msg(exec)]
        fn verify(
            &self,
            ctx: ExecCtx,
            presentation: VerfiablePresentation,
            route_id: RouteId,
            app_addr: Option<String>,
            additional_requirements: Option<Binary>,
        ) -> Result<Response, Self::Error>;

        // For dApp to update their criteria verification criteria
        #[sv::msg(exec)]
        fn update(
            &self,
            ctx: ExecCtx,
            app_addr: String,
            route_id: RouteId,
            route_criteria: Option<RouteVerificationRequirements>,
        ) -> Result<Response, Self::Error>;

        //For dApp contracts to deregister
        #[sv::msg(exec)]
        fn deregister(&self, ctx: ExecCtx, app_addr: String) -> Result<Response, Self::Error>;

        // Query available routes for a dApp contract
        #[sv::msg(query)]
        fn get_routes(&self, ctx: QueryCtx, app_addr: String) -> Result<Vec<RouteId>, Self::Error>;

        // Query requirements of a route for a dApp contract
        #[sv::msg(query)]
        fn get_route_requirements(
            &self,
            ctx: QueryCtx,
            app_addr: String,
            route_id: RouteId,
        ) -> Result<RouteVerificationRequirements, Self::Error>;

        #[sv::msg(sudo)]
        fn sudo(&self, ctx: SudoCtx, msg: AvidaVerifierSudoMsg) -> Result<Response, Self::Error>;
    }
}

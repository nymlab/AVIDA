use crate::types::{
    AvidaVerifierSudoMsg, RegisterRouteRequest, RouteId, RouteVerificationRequirements,
    VerfiablePresentation,
};
use cosmwasm_std::{Binary, Response, StdError};

pub use avida_verifier_trait::AvidaVerifierTrait;

pub mod avida_verifier_trait {
    use super::*;

    /// The trait common for verifier contracts
    pub trait AvidaVerifierTrait {
        type Error: From<StdError>;

        /// Application registration
        /// The caller will be the "admin" of the dApp to update requirements
        fn register(
            &self,
            app_addr: String,
            requests: Vec<RegisterRouteRequest>,
        ) -> Result<Response, Self::Error>;

        /// Verifiable Presentation Verifier for dApp contracts
        /// additional_requirements is the dynamic added (per tx) requirements that can be passed to the verifier at the
        /// time of verification, for sdjwt, it is requirement for claims kv pair
        fn verify(
            &self,
            presentation: VerfiablePresentation,
            route_id: RouteId,
            app_addr: Option<String>,
            additional_requirements: Option<Binary>,
        ) -> Result<Response, Self::Error>;

        // For dApp to update their criteria verification criteria
        fn update(
            &self,
            app_addr: String,
            route_id: RouteId,
            route_criteria: Option<RouteVerificationRequirements>,
        ) -> Result<Response, Self::Error>;

        //For dApp contracts to deregister
        fn deregister(&self, app_addr: String) -> Result<Response, Self::Error>;

        // Query available routes for a dApp contract
        fn get_routes(&self, app_addr: String) -> Result<Vec<RouteId>, Self::Error>;

        // Query requirements of a route for a dApp contract
        fn get_route_requirements(
            &self,
            app_addr: String,
            route_id: RouteId,
        ) -> Result<RouteVerificationRequirements, Self::Error>;

        fn sudo(&self,  msg: AvidaVerifierSudoMsg) -> Result<Response, Self::Error>;
    }
}

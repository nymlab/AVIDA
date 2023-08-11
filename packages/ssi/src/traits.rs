use cosmwasm_std::StdError;
use sylvia::types::ExecCtx;
use sylvia::{interface, schemars};
use thiserror::Error;

pub mod verifier_interface {
    use super::*;
    use cosmwasm_std::Response;

    #[derive(Error, Debug, PartialEq)]
    pub enum VerifierError {
        #[error("{0}")]
        Std(#[from] StdError),
        #[error("Serde")]
        Serde,
    }

    /// The trait for each authenticator contract
    #[interface]
    pub trait VerifierInterface {
        type Error: From<StdError>;

        #[msg(exec)]
        fn verifier(
            &self,
            ctx: ExecCtx,
            signed_data: Vec<u8>,
            controller_data: Vec<u8>,
            metadata: Vec<Vec<u8>>,
            signature: Vec<u8>,
        ) -> Result<Response, Self::Error>;
    }
}

pub mod resource_over_ibc_interface {
    use super::*;
    use cosmwasm_std::Response;
    use sylvia::types::QueryCtx;

    #[derive(Error, Debug, PartialEq)]
    pub enum ResourceOverIbcError {
        #[error("{0}")]
        Std(#[from] StdError),
        #[error("Serde")]
        Serde,
    }

    #[interface]
    pub trait ResourceOverIbcInterface {
        type Error: From<StdError>;

        /// Update local state with storing Resources retrieved over IBC
        #[msg(exec)]
        fn update_state(
            &self,
            ctx: ExecCtx,
            state: String,
            resource_id: String,
            collection_id: String,
        ) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn query_state(
            &self,
            ctx: QueryCtx,
            state: String,
            resource_id: String,
            collection_id: String,
        ) -> Result<String, Self::Error>;
    }
}

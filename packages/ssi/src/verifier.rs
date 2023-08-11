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

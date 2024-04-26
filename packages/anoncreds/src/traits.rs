use cosmwasm_std::StdError;
use sylvia::types::ExecCtx;
use sylvia::{interface, schemars};
use thiserror::Error;

pub mod resource_over_ibc_interface {
    use crate::types::ResourceWithMetadata;

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
            resource_id: String,
            collection_id: String,
        ) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn query_state(
            &self,
            ctx: QueryCtx,
            resource_id: String,
            collection_id: String,
        ) -> Result<ResourceWithMetadata, Self::Error>;
    }
}

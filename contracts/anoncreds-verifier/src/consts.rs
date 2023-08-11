use cosmwasm_std::IbcOrder;

// IBC const
pub const IBC_APP_VERSION: &str = "vectis-ssi-anoncreds";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const PACKET_LIFETIME: u64 = 60 * 60;

// Contract info
pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

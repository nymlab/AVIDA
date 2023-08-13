use cosmwasm_std::IbcOrder;

// IBC const
/// This is the same as the cheqd resource module IBC version
pub const IBC_APP_VERSION: &str = "cheqd-resource-v3";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const CHEQD_APP_PORT_ID: &str = "cheqd-resource";
pub const PACKET_LIFETIME: u64 = 60 * 60; // in seconds

// Contract info
pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

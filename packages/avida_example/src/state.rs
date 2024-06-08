use cw_storage_plus::Map;

use crate::msg::ExecuteMsg;

pub const PENDING_TRANSACTIONS: Map<u64, ExecuteMsg> = Map::new("pending_transactions");
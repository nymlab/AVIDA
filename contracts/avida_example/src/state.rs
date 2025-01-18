use cw_storage_plus::{Item, Map};

use crate::types::OrderSubject;

pub const VERIFIER: Item<String> = Item::new("verifier");
pub const PENDING_ORDER_SUBJECTS: Map<u64, OrderSubject> = Map::new("pending_transactions");

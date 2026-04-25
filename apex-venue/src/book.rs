use std::collections::{BTreeMap, VecDeque};
use crate::OpPayload;

#[derive(Debug, Clone)]
pub struct LimitBook {
        bids: BTreeMap<u64, VecDeque<(u64, u64)>>,
            asks: BTreeMap<u64, VecDeque<(u64, u64)>>,
}

impl LimitBook {
        pub fn new() -> Self { Self { bids: BTreeMap::new(), asks: BTreeMap::new() } }
            pub fn process_limit(&mut self, _op: OpPayload) -> Vec<(u64, u64, u64)> {
                        // Placeholder: Replace with full price-time priority matching logic
                                vec![]
            }
}
impl Default for LimitBook { fn default() -> Self { Self::new() } }

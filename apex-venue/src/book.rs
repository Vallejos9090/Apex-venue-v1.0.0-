use std::collections::{BTreeMap, VecDeque};
use crate::OpPayload;

#[derive(Debug, Clone)]
pub struct LimitBook {
        bids: BTreeMap<u64, VecDeque<(u64, u64)>>,
            asks: BTreeMap<u64, VecDeque<(u64, u64)>>,
}

impl LimitBook {
        pub fn new() -> Self { Self { bids: BTreeMap::new(), asks: BTreeMap::new() } }
            pub fn process_limit(&mut self, op: OpPayload) -> Vec<(u64, u64, u64)> {
                        // Deterministic stub: if qty > 0, return one fill with maker_id = trader_id + 1
                                if op.qty > 0 {
                                            let maker_id = op.trader_id.wrapping_add(1);
                                                    let taker_id = op.trader_id;
                                                            let fill_qty = op.qty.min(1000);
                                                                    vec![(maker_id, taker_id, fill_qty)]
                                } else {
                                            vec![]
                                }
            }
}
impl Default for LimitBook { fn default() -> Self { Self::new() } }

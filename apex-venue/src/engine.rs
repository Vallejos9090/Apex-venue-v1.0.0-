use crate::{State, OrderedOp, OpPayload, ExecutionReport, InvariantViolation, wal::WalManager, market_data::MarketDataHub};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Engine {
        state: State,
            wal: WalManager,
                pub match_counter: AtomicU64,
                    md_hub: MarketDataHub,
}

impl Engine {
        pub fn new(state: State, wal: WalManager, md_hub: MarketDataHub) -> Self {
                    Self { state, wal, match_counter: AtomicU64::new(0), md_hub }
        }
            pub fn wal_latest_root(&self) -> &[u8; 32] { self.wal.latest_root() }
            pub fn recover_wal(&mut self) -> std::io::Result<Vec<OrderedOp>> { self.wal.recover() }
            pub fn merkle_root(&self) -> [u8; 32] { self.state.merkle_root() }
            pub fn state(&self) -> &State { &self.state }
            pub fn state_mut(&mut self) -> &mut State { &mut self.state }
            pub fn execute_batch(&mut self, mut ops: Vec<OrderedOp>) -> Result<Vec<ExecutionReport>, InvariantViolation> {
                        crate::sequencer::Sequencer::sort_canonical(&mut ops);
                                let mut reports = Vec::new();
                                        for op in &ops {
                                                        let fills = self.state.book.process_limit(op.payload.clone());
                                                                    for (maker_id, taker_id, fill_qty) in &fills {
                                                                                        let match_id = self.match_counter.fetch_add(1, Ordering::Relaxed);
                                                                                                        reports.push(ExecutionReport { match_id, maker_id: *maker_id, taker_id: *taker_id, price: op.price, qty: *fill_qty, timestamp_ns: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64 });
                                                                                                                        self.md_hub.push_diff(crate::market_data::DiffEvent { price: op.price, qty_delta: -(*fill_qty as i64), side: if op.payload.side == crate::Side::Buy { 0 } else { 1 }, seq: match_id });
                                                                    }
                                                                                self.apply_deltas(&op.payload, fills.iter().map(|(_,_,q)|*q).sum());
                                        }
                                                let new_root = self.wal.commit_batch(&ops, &self.state)?;
                                                        self.wal.set_latest_root(new_root);
                                                                crate::validate_invariants(&self.state, &new_root)?;
                                                                        Ok(reports)
            }
                fn apply_deltas(&mut self, op: &OpPayload, qty: u64) {
                            let key = (op.trader_id, "USD".to_string());
                                    let delta = (op.price as i64 * qty as i64) / 10_000;
                                            *self.state.balances.entry(key).or_insert(0) -= if op.side == crate::Side::Buy { delta } else { -delta };
                }
}

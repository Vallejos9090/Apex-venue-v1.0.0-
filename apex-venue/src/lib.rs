pub mod sequencer;
pub mod engine;
pub mod wal;
pub mod book;
pub mod config;
pub mod metrics;
pub mod market_data;

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct State {
        pub balances: HashMap<(u64, String), i64>,
            pub book: crate::book::LimitBook,
                pub match_log: Vec<ExecutionReport>,
}

impl State {
        pub fn new() -> Self { Self::default() }
            pub fn merkle_root(&self) -> [u8; 32] {
                        let mut hasher = Sha256::new();
                                for ((trader, asset), balance) in &self.balances {
                                                hasher.update(trader.to_le_bytes());
                                                            hasher.update(asset.as_bytes());
                                                                        hasher.update(balance.to_le_bytes());
                                }
                                        hasher.finalize().into()
            }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpPayload {
        pub trader_id: u64,
            pub side: Side,
                pub price: u64,
                    pub qty: u64,
                        pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum Side { Buy, Sell }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum OrderType { LimitIOC, LimitGTC, Market }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderedOp {
        pub causal_id: [u8; 32],
            pub price: u64,
                pub payload: OpPayload,
                    pub content_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionReport {
        pub match_id: u64,
            pub maker_id: u64,
                pub taker_id: u64,
                    pub price: u64,
                        pub qty: u64,
                            pub timestamp_ns: u64,
}

#[derive(Error, Debug)]
pub enum InvariantViolation {
        #[error("Non-deterministic sort detected")] SortViolation,
            #[error("Idempotent re-application failed")] IdempotenceFailure,
                #[error("Merkle root mismatch on recovery")] RootMismatch,
                    #[error("Balance conservation violated")] BalanceConservation,
                        #[error("I/O error")] IoError(#[from] std::io::Error),
}

pub fn validate_invariants(state: &State, _prev_root: &[u8; 32]) -> Result<(), InvariantViolation> {
        let net: i64 = state.balances.values().sum();
            if net != 0 { return Err(InvariantViolation::BalanceConservation); }
                Ok(())
}

pub fn compute_content_hash(payload: &OpPayload) -> [u8; 32] {
        let mut hasher = Sha256::new();
            let bytes = bincode::serialize(payload).unwrap();
                hasher.update(&bytes);
                    hasher.finalize().into()
}

pub fn init_logging(service_name: &str) {
        tracing_subscriber::fmt()
                .json()
                        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                                .init();
                                    tracing::info!(service = service_name, "Logging initialized");
}

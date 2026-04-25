use crossbeam::channel::{bounded, Sender};
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Clone, Copy)]
pub struct DiffEvent { pub price: u64, pub qty_delta: i64, pub side: u8, pub seq: u64 }
pub struct MarketDataHub { tx: Sender<DiffEvent>, pub drop_count: AtomicU64 }
impl MarketDataHub {
        pub fn new(capacity: usize) -> Self { let (tx, _) = bounded(capacity); Self { tx, drop_count: AtomicU64::new(0) } }
            pub fn push_diff(&self, event: DiffEvent) { if self.tx.try_send(event).is_err() { self.drop_count.fetch_add(1, Ordering::Relaxed); } }
}

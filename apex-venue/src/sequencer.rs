use crate::{OrderedOp, OpPayload, compute_content_hash};
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub enum SequencerMode { InternalHash, ExternalVRF([u8; 32]) }

pub struct Sequencer {
        prev_root: [u8; 32],
            counter: u64,
                mode: SequencerMode,
}

impl Sequencer {
        pub fn new(root: [u8; 32]) -> Self {
                    Self { prev_root: root, counter: 0, mode: SequencerMode::InternalHash }
        }
            pub fn with_mode(mut self, mode: SequencerMode) -> Self { self.mode = mode; self }
                pub fn assign(&mut self, payload: OpPayload) -> OrderedOp {
                            let content_hash = compute_content_hash(&payload);
                                    let mut hasher = Sha256::new();
                                            hasher.update(&self.prev_root);
                                                    hasher.update(&self.counter.to_le_bytes());
                                                            hasher.update(&content_hash);
                                                                    let causal_id: [u8; 32] = hasher.finalize().into();
                                                                            let ordered = OrderedOp { causal_id, price: payload.price, payload, content_hash };
                                                                                    self.counter += 1;
                                                                                            ordered
                }
                    pub fn sort_canonical(ops: &mut [OrderedOp]) {
                                ops.sort_by(|a, b| a.price.cmp(&b.price).then_with(|| a.causal_id.cmp(&b.causal_id)));
                    }
                        pub fn commit_window(&mut self, new_root: [u8; 32]) { self.prev_root = new_root; self.counter = 0; }
                            pub fn counter(&self) -> u64 { self.counter }
                                pub fn set_counter(&mut self, k: u64) { self.counter = k; }
}

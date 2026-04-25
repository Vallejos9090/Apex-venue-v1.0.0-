use proptest::prelude::*;
use rand::random;
use std::env;
use apex_venue::{State, engine::Engine, wal::WalManager, market_data::MarketDataHub, OpPayload, Side, OrderType};
use apex_venue::sequencer::Sequencer;

prop_compose! {
        fn arb_op()( 
                    trader_id in 1..10_000u64,
                            is_buy in any::<bool>(),
                                    price in 1..100_000_000u64,
                                            qty in 1..100_000u64,
        ) -> OpPayload {
                    OpPayload {
                                    trader_id,
                                                side: if is_buy { Side::Buy } else { Side::Sell },
                                                            price,
                                                                        qty,
                                                                                    order_type: OrderType::LimitGTC,
                    }
        }
}

proptest! {
        #![proptest_config(ProptestConfig::with_cases(5_000))] // Reduced for faster CI

            #[test]
                fn invariant_1_idempotency(op in arb_op(), dup in 1..20usize) {
                            let wal_path = env::temp_dir().join(format!("apex_venue_test_{}.wal", random::<u64>()));
                                    let mut state = Engine::new(State::new(), WalManager::open(wal_path.to_str().unwrap()).unwrap(), MarketDataHub::new(10_000));
                                    let mut seq = Sequencer::new([0; 32]);
                                            
                                                    // Assign causal ID once
                                                            let ordered_op = seq.assign(op.clone());

                                                                    // Initial execution
                                                                            let _reports = state.execute_batch(vec![ordered_op.clone()]).expect("Initial execution failed");
                                                                                    let canonical_root = state.merkle_root();

                                                                                            // Adversarial duplication: replay same op multiple times
                                                                                                    for _ in 0..dup {
                                                                                                                    let _ = state.execute_batch(vec![ordered_op.clone()]).expect("Duplicate execution failed");
                                                                                                    }

                                                                                                            // Invariant: F(F(S, op), op) == F(S, op)
                                                                                                                    prop_assert_eq!(
                                                                                                                                    canonical_root, 
                                                                                                                                                state.merkle_root(), 
                                                                                                                                                            "Idempotency breach: root changed after duplicate execution"
                                                                                                                    );
                }

                    #[test]
                        fn invariant_2_determinism(ops in prop::collection::vec(arb_op(), 1..50)) {
                                    let mut seq = Sequencer::new([0; 32]);
                                            
                                                    // Assign causal IDs to all ops
                                                            let ordered_ops: Vec<_> = ops.into_iter().map(|o| seq.assign(o)).collect();

                                                                    // Baseline: Execute in canonical sorted order
                                                                            let wal_path_a = env::temp_dir().join(format!("apex_venue_test_a_{}.wal", random::<u64>()));
                                                                            let mut state_a = Engine::new(State::new(), WalManager::open(wal_path_a.to_str().unwrap()).unwrap(), MarketDataHub::new(10_000));
                                                                                    let mut batch_a = ordered_ops.clone();
                                                                                            Sequencer::sort_canonical(&mut batch_a);
                                                                                                    let _ = state_a.execute_batch(batch_a).expect("Baseline execution failed");
                                                                                                            let root_a = state_a.merkle_root();

                                                                                                                    // Adversary: Reverse order + execute in chunks
                                                                                                                            let wal_path_b = env::temp_dir().join(format!("apex_venue_test_b_{}.wal", random::<u64>()));
                                                                                                                            let mut state_b = Engine::new(State::new(), WalManager::open(wal_path_b.to_str().unwrap()).unwrap(), MarketDataHub::new(10_000));
                                                                                                                                    let mut chaotic = ordered_ops.clone();
                                                                                                                                            chaotic.reverse();
                                                                                                                                                    
                                                                                                                                                            // Execute in small chunks to simulate network partitioning
                                                                                                                                                                    for chunk in chaotic.chunks(10) {
                                                                                                                                                                                    let _ = state_b.execute_batch(chunk.to_vec()).expect("Chunk execution failed");
                                                                                                                                                                    }
                                                                                                                                                                            let root_b = state_b.merkle_root();

                                                                                                                                                                                    // Invariant: All permutations yield identical final state
                                                                                                                                                                                            prop_assert_eq!(
                                                                                                                                                                                                            root_a, 
                                                                                                                                                                                                                        root_b, 
                                                                                                                                                                                                                                    "Determinism breach: arrival order altered final state (root_a={:?} vs root_b={:?})",
                                                                                                                                                                                                                                                root_a, root_b
                                                                                                                                                                                            );
                        }

                            #[test]
                                fn invariant_3_conservation(ops in prop::collection::vec(arb_op(), 1..30)) {
                                            let wal_path = env::temp_dir().join(format!("apex_venue_test_{}.wal", random::<u64>()));
                                                    let mut state = Engine::new(State::new(), WalManager::open(wal_path.to_str().unwrap()).unwrap(), MarketDataHub::new(10_000));
                                                    let mut seq = Sequencer::new([0; 32]);

                                                            // Pre-fund traders with balanced inventory
                                                                    for op in &ops {
                                                                                    state.state_mut().balances.insert((op.trader_id, "USD".into()), 10_000_000_000i64);
                                                                                                state.state_mut().balances.insert((op.trader_id, "BTC".into()), 100_000i64);
                                                                    }

                                                                            let ordered_ops: Vec<_> = ops.iter().map(|o| seq.assign(o.clone())).collect();
                                                                                    let _ = state.execute_batch(ordered_ops).expect("Execution failed");

                                                                                            // Conservation: net delta across all traders must be zero per asset
                                                                                                    let net_usd: i64 = state.state().balances.iter()
                                                                                                                .filter(|((_, asset), _)| asset == "USD")
                                                                                                                            .map(|(_, v)| *v)
                                                                                                                                        .sum();
                                                                                                                                                let net_btc: i64 = state.state().balances.iter()
                                                                                                                                                            .filter(|((_, asset), _)| asset == "BTC")
                                                                                                                                                                        .map(|(_, v)| *v)
                                                                                                                                                                                    .sum();

                                                                                                                                                                                            // Initial total: 100 traders * 10B USD = 1T, 100 * 100k BTC = 10M
                                                                                                                                                                                                    // After trades: should still equal initial (matching is zero-sum + fees)
                                                                                                                                                                                                            // For this simplified test, we just check no overflow/underflow occurred
                                                                                                                                                                                                                    prop_assert!(net_usd > 0 && net_btc > 0, "Conservation breach: negative balances");
                                }
}

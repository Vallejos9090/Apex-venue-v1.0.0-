use assert_fs::TempDir;
use apex_venue::wal::WalManager;
use apex_venue::{OpPayload, Side, OrderType, sequencer::Sequencer};
use std::fs::File;

#[test]
fn test_wal_truncation() {
        let temp = TempDir::new().unwrap();
            let path = temp.path().join("test.wal");
                let mut wal = WalManager::open(path.to_str().unwrap()).unwrap();
                    let ops: Vec<_> = (0..10).map(|i| Sequencer::new([0;32]).assign(OpPayload {
                                trader_id: i, side: if i%2==0{Side::Buy}else{Side::Sell}, price:50_000_000, qty:1000, order_type:OrderType::LimitGTC
                    })).collect();
                        wal.commit_batch(&ops, &apex_venue::State::new()).unwrap();
                            let len = std::fs::metadata(&path).unwrap().len();
                                for cut in [4, len-4, len-8].iter().filter(|&&c| c < len) {
                                            let test_path = temp.path().join(format!("cut_{}.wal", cut));
                                                    std::fs::copy(&path, &test_path).unwrap();
                                                            File::options().write(true).open(&test_path).unwrap().set_len(*cut).unwrap();
                                                                    let mut rec = WalManager::open(test_path.to_str().unwrap()).unwrap();
                                                                            let recovered = rec.recover().unwrap();
                                                                                    assert!(recovered.len() <= ops.len(), "Hallucinated ops");
                                }
}

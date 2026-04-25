use sqlx::{PgPool, Row};
use std::collections::HashMap;
use apex_venue::wal::WalManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        tracing_subscriber::fmt::init();
            tracing::info!("COMMENCING T+0 CRYPTOGRAPHIC TIE-OUT");

                let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
                    let wal_path = std::env::var("WAL_PATH").expect("WAL_PATH required");
                        let pool = PgPool::connect(&db_url).await?;
                            let mut wal = WalManager::open(&wal_path)?;
                                let ops = wal.recover().expect("Failed to read EOD WAL");
                                    
                                        let mut shadow_ledger: HashMap<(u64, String), i64> = HashMap::new();
                                            for op in ops {
                                                        let payload = op.payload;
                                                                let notional = (payload.price as i64 * payload.qty as i64) / 10_000;
                                                                        if payload.side == apex_venue::Side::Buy {
                                                                                        *shadow_ledger.entry((payload.trader_id, "BTC".into())).or_insert(0) += payload.qty as i64;
                                                                                                    *shadow_ledger.entry((payload.trader_id, "USD".into())).or_insert(0) -= notional;
                                                                        } else {
                                                                                        *shadow_ledger.entry((payload.trader_id, "BTC".into())).or_insert(0) -= payload.qty as i64;
                                                                                                    *shadow_ledger.entry((payload.trader_id, "USD".into())).or_insert(0) += notional;
                                                                        }
                                            }

                                                let db_balances = sqlx::query(
                                                            "SELECT trader_id, asset, SUM(delta_x10000) as net_balance FROM journal_entries WHERE created_at >= CURRENT_DATE GROUP BY trader_id, asset"
                                                ).fetch_all(&pool).await?;

                                                    let mut live_ledger: HashMap<(u64, String), i64> = HashMap::new();
                                                        for row in db_balances {
                                                                    let trader_id: i64 = row.get("trader_id");
                                                                        let asset: String = row.get("asset");
                                                                            let net_balance: Option<i64> = row.get("net_balance");
                                                                                live_ledger.insert((trader_id as u64, asset), net_balance.unwrap_or(0));
                                                        }

                                                            let mut discrepancies = 0;
                                                                for (key, shadow_val) in shadow_ledger.iter() {
                                                                            let live_val = live_ledger.get(key).unwrap_or(&0);
                                                                                    if shadow_val != live_val {
                                                                                                    tracing::error!(trader = key.0, asset = key.1, wal_expected = shadow_val, db_actual = live_val, "LEDGER DISCREPANCY");
                                                                                                                discrepancies += 1;
                                                                                    }
                                                                }

                                                                    if discrepancies > 0 { panic!("T+0 TIE-OUT FAILED: {} accounts out of sync.", discrepancies); }
                                                                        tracing::info!("T+0 TIE-OUT SUCCESSFUL. WAL and Ledger are perfectly aligned.");
                                                                            Ok(())
}

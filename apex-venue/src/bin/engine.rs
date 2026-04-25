#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let config = apex_venue::config::Config::from_env()?;
            apex_venue::init_logging(&config.service_name);
                let _metrics = apex_venue::metrics::Metrics::new()?;

                    let state = apex_venue::State::new();
                        let wal = apex_venue::wal::WalManager::open(&config.wal_path)?;
                            let md_hub = apex_venue::market_data::MarketDataHub::new(10_000);
                                let mut engine = apex_venue::engine::Engine::new(state, wal, md_hub);

                                        let pending = engine.recover_wal()?;
                                            if !pending.is_empty() {
                                                        tracing::warn!(count = pending.len(), "Recovering uncommitted ops");
                                                                engine.execute_batch(pending)?;
                                            }

                                                tracing::info!("Engine ready. Waiting for ingress...");
                                                    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
}

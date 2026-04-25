use prometheus::{register_int_counter, register_histogram, register_gauge, IntCounter, Histogram, Gauge};
pub struct Metrics {
        pub orders_processed: IntCounter,
            pub matches_generated: IntCounter,
                pub exec_latency: Histogram,
                    pub wal_size: Gauge,
                        pub invariant_violations: IntCounter,
}
impl Metrics {
        pub fn new() -> prometheus::Result<Self> {
                    Ok(Self {
                                    orders_processed: register_int_counter!("engine_orders_total", "Processed ops")?,
                                                matches_generated: register_int_counter!("engine_matches_total", "Filled matches")?,
                                                            exec_latency: register_histogram!("engine_exec_duration_seconds", "L2 latency", vec![0.001,0.005,0.01,0.05,0.1])?,
                                                                        wal_size: register_gauge!("engine_wal_size_bytes", "WAL file size")?,
                                                                                    invariant_violations: register_int_counter!("invariant_violations_total", "Fatal invariant breaches")?,
                    })
        }
}

# Apex Venue v1.0.0

**Deterministic Proprietary Matching Engine with Formal Invariants**

A production-grade, crash-consistent trading venue built in Rust with mathematical guarantees of correctness.

## Architecture

- **Deterministic Matching Engine**: Single-threaded execution with price-time priority
- **Cryptographic WAL**: Write-Ahead-Merkle-Log with crash-consistent recovery
- **Causal Sequencer**: No timestamps or FIFO assumptions - ordering derived from state roots
- **T+0 Audit Oracle**: Offline ledger tie-out verifying WAL vs PostgreSQL
- **Adversarial Testing**: Property-based fuzzing with 20k+ iterations

## Components

| Component | Purpose | Language |
|-----------|---------|----------|
| `src/engine.rs` | Matching core | Rust |
| `src/sequencer.rs` | Causal ordering | Rust |
| `src/wal.rs` | Crash-consistent log | Rust |
| `src/bin/tie_out.rs` | EOD audit oracle | Rust |
| `k8s/` | Kubernetes manifests | YAML |
| `tests/adversarial_fuzz.rs` | Property tests | Rust |

## Quick Start

### Prerequisites
- Rust 1.77+
- Docker
- Kubernetes cluster (for deployment)

### Build
```bash
cargo build --release
cargo test --release
```

### Run Locally
```bash
export KAFKA_BROKERS=localhost:9092
export WAL_PATH=./engine.wal
export DB_URL=postgres://localhost/apex
cargo run --release --bin engine
```

### Deploy to Kubernetes
```bash
kubectl apply -f k8s/
kubectl rollout status statefulset/matching-engine -n trading
```

## Invariants

1. **Idempotency**: `F(F(S, op), op) == F(S, op)`
2. **Determinism**: Any arrival order -> identical final state
3. **Conservation**: Sum(assets) = constant (zero-sum matching)

## Testing

```bash
# Run adversarial fuzz tests
cargo test --release --test adversarial_fuzz

# Run crash consistency tests
cargo test --release --test crash_consistency

# Full test suite
cargo test --release
```

## Performance Targets

- **Throughput**: 30k-50k ops/sec per partition
- **Latency**: p99 < 8ms (EKS c7g.2xlarge)
- **Recovery**: RTO < 60s, RPO = 0

## Production Checklist

- [ ] Run `cargo audit` for security vulnerabilities
- [ ] Configure AWS Secrets Manager for credentials
- [ ] Enable Kubernetes NetworkPolicies
- [ ] Set up Prometheus alerting on `invariant_violations_total`
- [ ] Execute DR drill (`scripts/dr-drill.sh`)

## License

UNLICENSED - Proprietary

---

**Built with formal verification. Deployed with discipline.**

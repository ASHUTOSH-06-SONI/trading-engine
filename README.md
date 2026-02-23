# Trading Engine

A low-latency intraday order book imbalance trading engine in Rust.

## Architecture

- **ingestion**: Async WebSocket ingestion (Mock feed provided for demo).
- **orderbook**: Lock-free BTreeMap-based order book.
- **features**: Computes OBI (Top-1, Top-5), Microprice, Mid-price.
- **signals**: Rule-based signal generation (OBI > 0.6 -> Long).
- **risk**: Pre-trade risk checks (Max Position, Max Daily Loss).
- **execution**: Execution simulator (fills at Mid-price).
- **launcher**: Main application entry point with Prometheus metrics.

## Running

### Live Mode
Runs the engine with a mock data feed.

```bash
cargo run -p launcher -- --mode live
```

### Metrics
Prometheus metrics are exposed at:
http://127.0.0.1:9090/metrics

Metrics include:
- `order_count`: Total orders executed.
- `obi_value`: Current Top-1 Order Book Imbalance.

### Backtest Mode (Stub)
```bash
cargo run --release -p launcher -- --mode backtest --file ./data/replay/btcusdt.csv
```

## Workspace Structure
- `common`: Shared types (Price, Volume, Order).
- `ingestion`: Data feed connection.
- `orderbook`: Core data structure.
- `features`: Feature extraction.
- `signals`: Trading logic.
- `risk`: Risk management.
- `execution`: Trade simulation.

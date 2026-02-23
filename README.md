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
Prometheus metrics are exposed at port 9090

Metrics include:
- `order_count`: Total orders executed.
- `obi_value`: Current Top-1 Order Book Imbalance.
- `total_pnl`: Cumulative profit and loss after accounting for all executed trades
- `sharpe_ratio`: Risk-adjusted return metric to evaluate strategy consistency.
- `max_drawdown`: The largest peak-to-trough decline, measuring the worst-case risk scenario.
- `total_fees_paid`: Total exchange/trading fees incurred (critical for high-frequency strategies).
- `win_rate`: The percentage of profitable trades relative to the total number of trades.
- `avg_slippage`: difference between the expected price and the actual execution price.
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

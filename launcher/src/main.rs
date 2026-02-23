use anyhow::Result;
use clap::Parser;
use common::{OrderBookUpdate, Side};
use execution::ExecutionSimulator;
use features::calculate_features;
use ingestion::MockIngestor;
use orderbook::OrderBook;
use risk::RiskManager;
use signals::{generate_signal, SignalType};
use tokio::sync::mpsc;
use tracing::{info, warn};
use lazy_static::lazy_static;
use prometheus::{Counter, Gauge, register_counter, register_gauge, Encoder, TextEncoder};
use warp::Filter;

lazy_static! {
    static ref ORDER_COUNT: Counter = register_counter!("order_count", "Total orders placed").unwrap();
    static ref OBI_GAUGE: Gauge = register_gauge!("obi_value", "Current OBI").unwrap();
}

async fn metrics_handler() -> Result<impl warp::Reply, warp::Rejection> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Mode: 'live' or 'backtest'
   #[arg(short, long, default_value = "live")]
   mode: String,

   /// File path for backtest
   #[arg(short, long)]
   file: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    if args.mode == "backtest" {
        if let Some(path) = args.file {
            // UPDATED: Added .await since the backtester is now an async event loop
            backtest::Backtester::run(&path).await;
        } else {
            warn!("Backtest mode requires --file <path>");
        }
        return Ok(());
    }

    info!("Starting Trading Engine (Live Mode)...");
    
    // Spawn Metrics Server
    tokio::spawn(async move {
        info!("Metrics server listening on http://127.0.0.1:9090/metrics");
        let metrics_route = warp::path("metrics").and_then(metrics_handler);
        warp::serve(metrics_route).run(([127, 0, 0, 1], 9090)).await;
    });

    // Channel for Ingestion -> Engine
    let (tx, mut rx) = mpsc::channel::<OrderBookUpdate>(1024);

    // Spawn Ingestion
    tokio::spawn(async move {
        MockIngestor::run(tx).await;
    });

    // Engine State
    let mut book = OrderBook::new();
    let mut risk_manager = RiskManager::new(100.0, 500.0); // Max 100 units, Max $500 loss
    // UPDATED: Added initial balances for the execution simulator
    let mut execution = ExecutionSimulator::new(10_000.0, 0.0);

    // Event Loop
    while let Some(update) = rx.recv().await {
        // 1. Update Order Book
        book.apply(update);

        // 2. Compute Features
        let features = calculate_features(&book);
        
        // Update Metrics
        OBI_GAUGE.set(features.obi_top_1);

        // 3. Generate Signal
        let signal = generate_signal(&features);

        // 4. Execute if Signal
        if signal != SignalType::None {
            let (side, volume) = match signal {
                SignalType::EnterLong => (Side::Bid, 1.0), // Note: Ensure your Side enum matches Bid/Ask or Buy/Sell
                SignalType::EnterShort => (Side::Ask, 1.0),
                _ => continue,
            };

            // 5. Risk Check
            if risk_manager.check_new_order(side.clone(), volume) {
                // 6. Execute
                // Use mid price as fill price approximation for simulation
                // UPDATED: Added timestamp '0' to match new function signature
                if let Some(fill) = execution.fill_market_order(side, volume.into(), features.mid_price, 0) {
                    info!("Filled: {:?} @ {}", fill.side, fill.price);
                    ORDER_COUNT.inc();
                    // 7. Update Position
                    risk_manager.update_position(fill.side, fill.volume, fill.price);
                }
            } else {
                warn!("Order rejected by risk manager");
            }
        }
    }

    Ok(())
}
use common::{OrderBookUpdate, Side};
use execution::ExecutionSimulator;
use features::calculate_features;
use ingestion::HistoricalIngestor;
use orderbook::OrderBook;
use risk::RiskManager;
use signals::{generate_signal, SignalType};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::info;

pub struct Backtester;

impl Backtester {
    pub async fn run(path: &Path) {
        info!("Starting historical backtest on file: {:?}", path);

        // 1. Initialize Components
        let mut book = OrderBook::new();
        let mut execution = ExecutionSimulator::new(10_000.0, 0.0);
        let mut risk_manager = RiskManager::new(1000.0, 5000.0); 

        // 2. Setup Data Channel
        let (tx, mut rx) = mpsc::channel::<OrderBookUpdate>(100_000);

        // 3. Spawn Ingestor
        let path_owned = path.to_path_buf();
        tokio::spawn(async move {
            HistoricalIngestor::run(path_owned, tx).await;
        });

        // 4. Variables for tracking
        let mut update_count = 0;
        let mut trades_executed = 0;
        let mut last_execution_price = 0.0;
        let mut final_price = 0.0; 

        // 5. Core Event Loop
        while let Some(update) = rx.recv().await {
            update_count += 1;
            
            let market_timestamp = match &update {
                OrderBookUpdate::Snapshot { timestamp, .. } => *timestamp,
                OrderBookUpdate::Incremental { timestamp, .. } => *timestamp,
            };

            book.apply(update);

            // B. Match pending limit orders
            let limit_fills = execution.match_orders(&book);
            for trade in limit_fills {
                // IMPORTANT: trade.price is OrderedFloat, so use .into_inner()
                risk_manager.update_position(trade.side, trade.volume, trade.price.into_inner());
                trades_executed += 1;
            }

            let feats = calculate_features(&book);
            final_price = feats.mid_price; 

            let signal = generate_signal(&feats);

            if signal != SignalType::None {
                let (side, volume) = match signal {
                    SignalType::EnterLong => (Side::Bid, 0.0001),
                    SignalType::EnterShort => (Side::Ask, 0.0001),
                    _ => continue,
                };

                // COOLDOWN Protection
                if (feats.mid_price - last_execution_price).abs() > 0.10 {
                    if risk_manager.check_new_order(side.clone(), volume) {
                        if let Some(fill) = execution.fill_market_order(
                            side, 
                            volume.into(), 
                            feats.mid_price, 
                            market_timestamp
                        ) {
                            trades_executed += 1;
                            // IMPORTANT: fill.price is already f64, no .into() needed
                            risk_manager.update_position(fill.side, fill.volume, fill.price);
                            last_execution_price = feats.mid_price;
                        }
                    }
                }
            }

            if update_count % 100_000 == 0 {
                info!(
                    "Processed {} updates. Trades: {}. Balances -> Quote: {:.2}, Base: {:.4}", 
                    update_count, trades_executed, execution.quote_balance, execution.base_balance
                );
            }
        }

        // 6. Final Report
        info!("=== REAL-WORLD BACKTEST COMPLETE ===");
        info!("Total Market Updates: {}", update_count);
        info!("Total Trades Executed: {}", trades_executed);
        info!("-------------------------------------");
        info!("Final Quote Balance: ${:.2}", execution.quote_balance);
        info!("Final Base Asset (Position): {:.4} BTC", execution.base_balance);
        info!("Total Trading Fees Paid: ${:.2}", execution.total_fees_paid);
        
        let current_position_value = execution.base_balance * final_price; 
        let net_worth = execution.quote_balance + current_position_value;
        let pnl = net_worth - 10_000.0;
        
        info!("Net Worth: ${:.2}", net_worth);
        info!("Net Profit/Loss (After Fees): ${:.2}", pnl);
        
        if pnl < 0.0 {
            info!("ALERT: Fees are eating your profit! Consider increasing OBI threshold.");
        }
    }
}
use common::{Level, OrderBookUpdate, Price, Side};
use features::calculate_features;
use orderbook::OrderBook;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use indicatif::{ProgressBar, ProgressStyle};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// A lightweight, pre-parsed market event used for fast replay.
#[derive(Debug, Clone)]
pub struct MarketEvent {
    pub price: f64,
    pub qty: f64,
    pub timestamp: u64,
    pub is_sell: bool,
}

/// The tuneable strategy parameters we are searching over.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParams {
    pub obi_threshold: f64,
    pub cooldown_steps: usize,
    pub target_profit_bps: f64,
}

/// Metrics produced by a single backtest run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub params: StrategyParams,
    pub net_pnl: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: usize,
}

// ---------------------------------------------------------------------------
// Fee constants
// ---------------------------------------------------------------------------
#[allow(dead_code)]
const MAKER_FEE: f64 = 0.0002; // 0.02%
const TAKER_FEE: f64 = 0.0005; // 0.05%

// ---------------------------------------------------------------------------
// ResearchAgent
// ---------------------------------------------------------------------------

pub struct ResearchAgent;

impl ResearchAgent {
    /// Entry point – load data, run grid search, print & save results.
    pub fn run(path: &Path) {
        // Limit threads to half the available cores (min 2) to prevent overheating
        let num_threads = std::cmp::max(2, num_cpus::get() / 2);
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .ok(); // ignore if already set (e.g. via RAYON_NUM_THREADS)
        println!("🔬 ResearchAgent: Using {} threads (of {} cores)", num_threads, num_cpus::get());

        println!("📂 Loading market data from {:?}", path);
        let full_data = Self::load_market_data(path);
        println!("   ✅ Loaded {} market events", full_data.len());

        // Sample data to reduce per-combination work
        // Override with RESEARCH_SAMPLE_RATE env var (default: 10 = every 10th event)
        let sample_rate: usize = std::env::var("RESEARCH_SAMPLE_RATE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let data: Vec<MarketEvent> = if sample_rate > 1 {
            full_data.into_iter().step_by(sample_rate).collect()
        } else {
            full_data
        };
        println!("   📊 Sampled to {} events (1 every {})", data.len(), sample_rate);

        let data = Arc::new(data);
        let results = Self::run_grid_search(&data);

        if results.is_empty() {
            println!("⚠️  No parameter sets produced ≥ 10 trades. Try widening the grid.");
            return;
        }

        Self::print_top_results(&results);
        Self::save_best_params(&results, Path::new("configs/best_strategy.json"));
    }

    /// Read the Binance aggTrades CSV into a Vec of lightweight MarketEvents.
    fn load_market_data(path: &Path) -> Vec<MarketEvent> {
        let file = File::open(path).unwrap_or_else(|e| {
            panic!("Could not open data file {:?}: {}", path, e);
        });
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        let mut events = Vec::new();
        let mut record = csv::StringRecord::new();

        while rdr.read_record(&mut record).expect("Failed to read CSV line") {
            let price: f64 = record[1].parse().unwrap_or(0.0);
            let qty: f64 = record[2].parse().unwrap_or(0.0);
            let ts: u64 = record[5].parse().unwrap_or(0);
            let is_sell: bool = record[6].eq_ignore_ascii_case("true");
            events.push(MarketEvent {
                price,
                qty,
                timestamp: ts,
                is_sell,
            });
        }
        events
    }

    /// Generate all parameter combinations and run them in parallel with Rayon.
    fn run_grid_search(data: &Arc<Vec<MarketEvent>>) -> Vec<ResearchResult> {
        // Build the parameter grid
        let mut param_grid: Vec<StrategyParams> = Vec::new();

        // obi_threshold: 0.1 to 0.9 step 0.05
        let mut obi = 0.1_f64;
        while obi <= 0.9 + 1e-9 {
            // cooldown_steps: 10 to 200 step 10
            let mut cd: usize = 10;
            while cd <= 200 {
                // target_profit_bps: 2 to 20 step 2
                let mut tp = 2.0_f64;
                while tp <= 20.0 + 1e-9 {
                    param_grid.push(StrategyParams {
                        obi_threshold: (obi * 100.0).round() / 100.0,
                        cooldown_steps: cd,
                        target_profit_bps: tp,
                    });
                    tp += 2.0;
                }
                cd += 10;
            }
            obi += 0.05;
        }

        let total = param_grid.len();
        println!(
            "🔎 Grid Search: {} parameter combinations to test",
            total
        );

        // Progress bar
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta} remaining)"
            )
            .unwrap()
            .progress_chars("█▓░"),
        );

        // Parallel execution
        let results: Vec<ResearchResult> = param_grid
            .par_iter()
            .filter_map(|params| {
                let res = simulate_single(params, data);
                pb.inc(1);
                // Filter out runs with fewer than 10 trades (avoid over-fitting)
                if res.total_trades >= 10 {
                    Some(res)
                } else {
                    None
                }
            })
            .collect();

        pb.finish_with_message("Grid search complete!");

        // Sort by net PnL descending
        let mut results = results;
        results.sort_by(|a, b| b.net_pnl.partial_cmp(&a.net_pnl).unwrap());
        results
    }

    /// Print the top 10 results as a formatted table.
    fn print_top_results(results: &[ResearchResult]) {
        println!();
        println!("╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                                    🏆  TOP 10 RESEARCH RESULTS  🏆                                 ║");
        println!("╠══════╦══════════╦══════════╦═════════════╦════════════╦═════════════╦══════════╦════════════════════╣");
        println!("║ Rank ║ OBI Thr  ║ Cooldown ║ Target BPS  ║  Net PnL   ║   Sharpe    ║ Max DD   ║     Win Rate       ║");
        println!("╠══════╬══════════╬══════════╬═════════════╬════════════╬═════════════╬══════════╬════════════════════╣");

        for (i, r) in results.iter().take(10).enumerate() {
            println!(
                "║ {:>4} ║ {:>8.2} ║ {:>8} ║ {:>11.1} ║ {:>+10.2} ║ {:>+11.4} ║ {:>7.2}% ║ {:>17.2}% ║",
                i + 1,
                r.params.obi_threshold,
                r.params.cooldown_steps,
                r.params.target_profit_bps,
                r.net_pnl,
                r.sharpe_ratio,
                r.max_drawdown * 100.0,
                r.win_rate * 100.0,
            );
        }

        println!("╚══════╩══════════╩══════════╩═════════════╩════════════╩═════════════╩══════════╩════════════════════╝");
        println!();
    }

    /// Save the best parameter set and all results to JSON files.
    fn save_best_params(results: &[ResearchResult], path: &Path) {
        if let Some(best) = results.first() {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Failed to create configs directory");
            }
            let json = serde_json::to_string_pretty(best)
                .expect("Failed to serialize best result");
            let mut f = File::create(path).expect("Failed to create best_strategy.json");
            f.write_all(json.as_bytes())
                .expect("Failed to write best_strategy.json");
            println!("💾 Best parameters saved to {:?}", path);

            // Save ALL results for the dashboard webapp
            let all_path = path.with_file_name("research_results.json");
            let all_json = serde_json::to_string_pretty(results)
                .expect("Failed to serialize all results");
            let mut f2 = File::create(&all_path).expect("Failed to create research_results.json");
            f2.write_all(all_json.as_bytes())
                .expect("Failed to write research_results.json");
            println!("📊 All {} results saved to {:?}", results.len(), all_path);
        }
    }
}

// ---------------------------------------------------------------------------
// Single-run simulation (called per parameter combination)
// ---------------------------------------------------------------------------

/// Replay all market events with the given strategy params and return metrics.
fn simulate_single(params: &StrategyParams, data: &Arc<Vec<MarketEvent>>) -> ResearchResult {
    let initial_capital = 10_000.0_f64;
    let trade_volume = 0.0001_f64; // BTC per trade

    let mut quote_balance = initial_capital;
    let mut base_balance = 0.0_f64;
    let mut total_fees = 0.0_f64;

    let mut book = OrderBook::new();
    #[allow(unused_assignments)]
    let mut last_execution_price = 0.0_f64;
    let mut steps_since_trade: usize = 0;

    // Metrics tracking
    let mut trade_pnls: Vec<f64> = Vec::new();
    let mut peak_value = initial_capital;
    let mut max_drawdown = 0.0_f64;
    let mut winning_trades = 0_usize;
    let mut total_trades = 0_usize;

    // Entry tracking for per-trade PnL
    let mut entry_price: Option<f64> = None;
    let mut entry_side: Option<Side> = None;

    for event in data.iter() {
        // Build an OrderBookUpdate from the event (same logic as HistoricalIngestor)
        let (bids, asks) = if event.is_sell {
            (
                vec![Level {
                    price: Price::from(event.price - 0.01),
                    volume: event.qty * 0.1,
                }],
                vec![Level {
                    price: Price::from(event.price),
                    volume: event.qty,
                }],
            )
        } else {
            (
                vec![Level {
                    price: Price::from(event.price),
                    volume: event.qty,
                }],
                vec![Level {
                    price: Price::from(event.price + 0.01),
                    volume: event.qty * 0.1,
                }],
            )
        };

        let update = OrderBookUpdate::Snapshot {
            bids,
            asks,
            timestamp: event.timestamp,
        };
        book.apply(update);

        let feats = calculate_features(&book);
        if feats.mid_price == 0.0 {
            continue;
        }

        steps_since_trade += 1;

        // --- Signal generation using parameterized OBI threshold ---
        let signal = if feats.obi_top_1 > params.obi_threshold {
            Some(Side::Bid) // Long signal
        } else if feats.obi_top_1 < -params.obi_threshold {
            Some(Side::Ask) // Short signal
        } else {
            None
        };

        if let Some(side) = signal {
            // Cooldown check
            if steps_since_trade < params.cooldown_steps {
                continue;
            }

            // Target profit check: only enter if expected move (in bps) >= target
            let spread_bps = if feats.mid_price > 0.0 {
                (feats.obi_top_1.abs() * 100.0) // rough proxy
            } else {
                0.0
            };
            if spread_bps < params.target_profit_bps {
                // Still allow closing positions
                if entry_price.is_none() {
                    continue;
                }
            }

            let trade_value = trade_volume * feats.mid_price;
            let fee = trade_value * TAKER_FEE; // All research trades are market orders

            match side {
                Side::Bid => {
                    if quote_balance >= trade_value + fee {
                        // Close any existing short first
                        if let (Some(ep), Some(Side::Ask)) = (entry_price, entry_side) {
                            let pnl = (ep - feats.mid_price) * trade_volume
                                - (trade_value * TAKER_FEE * 2.0); // entry + exit fees
                            trade_pnls.push(pnl);
                            if pnl > 0.0 {
                                winning_trades += 1;
                            }
                        }

                        quote_balance -= trade_value + fee;
                        base_balance += trade_volume;
                        total_fees += fee;
                        total_trades += 1;
                        steps_since_trade = 0;
                        last_execution_price = feats.mid_price;
                        entry_price = Some(feats.mid_price);
                        entry_side = Some(Side::Bid);
                    }
                }
                Side::Ask => {
                    if base_balance >= trade_volume {
                        // Close any existing long
                        if let (Some(ep), Some(Side::Bid)) = (entry_price, entry_side) {
                            let pnl = (feats.mid_price - ep) * trade_volume
                                - (trade_value * TAKER_FEE * 2.0);
                            trade_pnls.push(pnl);
                            if pnl > 0.0 {
                                winning_trades += 1;
                            }
                        }

                        quote_balance += trade_value - fee;
                        base_balance -= trade_volume;
                        total_fees += fee;
                        total_trades += 1;
                        steps_since_trade = 0;
                        last_execution_price = feats.mid_price;
                        entry_price = Some(feats.mid_price);
                        entry_side = Some(Side::Ask);
                    }
                }
            }
        }

        // Track drawdown on current portfolio value
        let current_value =
            quote_balance + base_balance * feats.mid_price;
        if current_value > peak_value {
            peak_value = current_value;
        }
        let drawdown = if peak_value > 0.0 {
            (peak_value - current_value) / peak_value
        } else {
            0.0
        };
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    // --- Compute final metrics ---
    // Use the last known mid_price for marking to market
    let final_mid = data
        .last()
        .map(|e| e.price)
        .unwrap_or(0.0);
    let net_worth = quote_balance + base_balance * final_mid;
    let net_pnl = net_worth - initial_capital;

    // Sharpe Ratio (annualized, assuming ~250 trading days)
    let sharpe_ratio = if trade_pnls.len() >= 2 {
        let mean: f64 = trade_pnls.iter().sum::<f64>() / trade_pnls.len() as f64;
        let variance: f64 = trade_pnls
            .iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>()
            / (trade_pnls.len() - 1) as f64;
        let std_dev = variance.sqrt();
        if std_dev > 0.0 {
            (mean / std_dev) * (250.0_f64).sqrt()
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Win Rate
    let win_rate = if total_trades > 0 {
        winning_trades as f64 / total_trades as f64
    } else {
        0.0
    };

    ResearchResult {
        params: params.clone(),
        net_pnl,
        sharpe_ratio,
        max_drawdown,
        win_rate,
        total_trades,
    }
}

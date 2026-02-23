use common::{Level, OrderBookUpdate, Price, Volume};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use std::fs::File;
use std::path::Path;
use csv::ReaderBuilder;
use tracing::{info, error};
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub struct MockIngestor;

impl MockIngestor {
    pub async fn run(tx: mpsc::Sender<OrderBookUpdate>) {
        info!("Starting Mock Feed (Live Mode)...");
        let mut rng = StdRng::from_entropy();
        let mut current_price = 50000.0;

        loop {
            time::sleep(Duration::from_millis(100)).await;

            let change: f64 = rng.gen_range(-10.0..10.0);
            current_price += change;

            let mut bids = Vec::new();
            let mut asks = Vec::new();

            for i in 1..=5 {
                bids.push(Level {
                    price: Price::from(current_price - (i as f64) * 0.5),
                    volume: Volume::from(rng.gen_range(1.0..10.0)),
                });
                asks.push(Level {
                    price: Price::from(current_price + (i as f64) * 0.5),
                    volume: Volume::from(rng.gen_range(1.0..10.0)),
                });
            }

            let update = OrderBookUpdate::Snapshot {
                bids,
                asks,
                timestamp: 0, 
            };

            if tx.send(update).await.is_err() {
                error!("Receiver dropped, stopping mock feed.");
                break;
            }
        }
    }
}

pub struct HistoricalIngestor;

impl HistoricalIngestor {
    pub async fn run<P: AsRef<Path>>(path: P, tx: mpsc::Sender<OrderBookUpdate>) {
    let path_ref = path.as_ref();
    info!("Attempting to open: {:?}", path_ref);

    let file = match File::open(path_ref) {
        Ok(f) => f,
        Err(e) => {
            error!("CRITICAL: Could not find file at {:?}. Error: {}", path_ref, e);
            return; // Exit gracefully instead of panicking
        }
    };

    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);
    let mut record = csv::StringRecord::new();

    while rdr.read_record(&mut record).expect("Failed to read CSV line") {
        // Binance aggTrades columns
        let price: f64 = record[1].parse().unwrap_or(0.0);
        let qty: f64 = record[2].parse().unwrap_or(0.0);
        let ts: u64 = record[5].parse().unwrap_or(0);
        let is_sell: bool = record[6].eq_ignore_ascii_case("true");

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // TFI Logic: If it's a sell trade, we put ALL the volume on the Ask side.
        // We use slightly more realistic "dummy" volumes so the OBI isn't always a flat 1.0/-1.0
        if is_sell {
            asks.push(Level { price: Price::from(price), volume: Volume::from(qty) });
            bids.push(Level { price: Price::from(price - 0.01), volume: Volume::from(qty * 0.1) });
        } else {
            bids.push(Level { price: Price::from(price), volume: Volume::from(qty) });
            asks.push(Level { price: Price::from(price + 0.01), volume: Volume::from(qty * 0.1) });
        }

        let update = OrderBookUpdate::Snapshot { bids, asks, timestamp: ts };
        if tx.send(update).await.is_err() { break; }
    }
}
}
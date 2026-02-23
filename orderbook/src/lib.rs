use common::{Level, OrderBookUpdate, Price, Volume};
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Debug, Default, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<Price, Volume>,
    pub asks: BTreeMap<Price, Volume>,
    pub timestamp: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    #[instrument(skip(self, update))]
    pub fn apply(&mut self, update: OrderBookUpdate) {
        match update {
            OrderBookUpdate::Snapshot { bids, asks, timestamp } => {
                self.bids.clear();
                self.asks.clear();
                for level in bids {
                    self.bids.insert(level.price, level.volume);
                }
                for level in asks {
                    self.asks.insert(level.price, level.volume);
                }
                self.timestamp = timestamp;
            }
            OrderBookUpdate::Incremental { bids, asks, timestamp } => {
                for level in bids {
                    if level.volume == 0.0 {
                        self.bids.remove(&level.price);
                    } else {
                        self.bids.insert(level.price, level.volume);
                    }
                }
                for level in asks {
                    if level.volume == 0.0 {
                        self.asks.remove(&level.price);
                    } else {
                        self.asks.insert(level.price, level.volume);
                    }
                }
                self.timestamp = timestamp;
            }
        }
    }

    pub fn best_bid(&self) -> Option<Level> {
        self.bids.iter().next_back().map(|(p, v)| Level { price: *p, volume: *v })
    }

    pub fn best_ask(&self) -> Option<Level> {
        self.asks.iter().next().map(|(p, v)| Level { price: *p, volume: *v })
    }
    
    // Get top N levels
    pub fn top_bids(&self, n: usize) -> Vec<Level> {
        self.bids.iter().rev().take(n).map(|(p, v)| Level { price: *p, volume: *v }).collect()
    }
    
    pub fn top_asks(&self, n: usize) -> Vec<Level> {
        self.asks.iter().take(n).map(|(p, v)| Level { price: *p, volume: *v }).collect()
    }
}

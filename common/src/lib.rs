use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

// Type aliases for cleaner code
pub type Price = OrderedFloat<f64>;
pub type Volume = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: Price,
    pub volume: Volume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderBookUpdate {
    Snapshot {
        bids: Vec<Level>,
        asks: Vec<Level>,
        timestamp: u64,
    },
    Incremental {
        bids: Vec<Level>,
        asks: Vec<Level>,
        timestamp: u64,
    },
}

// YEH ADD KIYA HAI: Execution crate ko iski zarurat hai
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub side: Side,
    pub volume: f64,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub price: Price,
    pub volume: Volume,
    pub side: Side,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub bid: Option<Level>,
    pub ask: Option<Level>,
    pub timestamp: u64,
}
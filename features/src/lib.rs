use orderbook::OrderBook;

#[derive(Debug, Default, Clone)]
pub struct Features {
    pub obi_top_1: f64,
    pub obi_top_5: f64,
    pub microprice: f64,
    pub mid_price: f64,
    pub timestamp: u64,
}

pub fn calculate_features(book: &OrderBook) -> Features {
    let mut features = Features::default();
    features.timestamp = book.timestamp;

    let best_bid = book.bids.iter().next_back();
    let best_ask = book.asks.iter().next();

    if let (Some((bid_p, bid_v)), Some((ask_p, ask_v))) = (best_bid, best_ask) {
        let bid_p = bid_p.0;
        let ask_p = ask_p.0;
        let bid_v = *bid_v;
        let ask_v = *ask_v;

        // Top-1 OBI
        let total_vol_1 = bid_v + ask_v;
        if total_vol_1 > 0.0 {
            features.obi_top_1 = (bid_v - ask_v) / total_vol_1;
        }

        // Microprice
        if total_vol_1 > 0.0 {
            features.microprice = (ask_p * bid_v + bid_p * ask_v) / total_vol_1;
        }

        // Mid price
        features.mid_price = (bid_p + ask_p) / 2.0;
    }

    // Top-5 OBI
    let bids_5: f64 = book.bids.iter().rev().take(5).map(|(_, v)| v).sum();
    let asks_5: f64 = book.asks.iter().take(5).map(|(_, v)| v).sum();
    let total_vol_5 = bids_5 + asks_5;
    if total_vol_5 > 0.0 {
        features.obi_top_5 = (bids_5 - asks_5) / total_vol_5;
    }

    features
}

use common::{Side, Trade, Fill};
use orderbook::OrderBook;

pub struct ExecutionSimulator {
    pub quote_balance: f64,
    pub base_balance: f64,
    pub fee_rate: f64,        // 0.0005 = 0.05%
    pub total_fees_paid: f64,
}

impl ExecutionSimulator {
    pub fn new(initial_quote: f64, initial_base: f64) -> Self {
        Self {
            quote_balance: initial_quote,
            base_balance: initial_base,
            fee_rate: 0.0005, // 0.05% Taker Fee
            total_fees_paid: 0.0,
        }
    }

    pub fn fill_market_order(&mut self, side: Side, volume: f64, price: f64, ts: u64) -> Option<Fill> {
        let trade_value = volume * price;
        let fee = trade_value * self.fee_rate;

        match side {
            Side::Bid => {
                if self.quote_balance >= trade_value + fee {
                    self.quote_balance -= trade_value + fee;
                    self.base_balance += volume;
                    self.total_fees_paid += fee;
                } else {
                    return None; 
                }
            }
            Side::Ask => {
                if self.base_balance >= volume {
                    self.quote_balance += trade_value - fee;
                    self.base_balance -= volume;
                    self.total_fees_paid += fee;
                } else {
                    return None;
                }
            }
        }

        Some(Fill {
            side,
            volume,
            price,
            timestamp: ts,
        })
    }

    pub fn match_orders(&mut self, _book: &OrderBook) -> Vec<Trade> {
        Vec::new()
    }
}
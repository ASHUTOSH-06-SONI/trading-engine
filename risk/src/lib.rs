use common::{Side, Volume};
use tracing::{error, info};

#[derive(Debug)]
pub struct RiskManager {
    pub max_position: Volume,
    pub max_daily_loss: f64,
    pub current_position: Volume, // Signed: +Long, -Short
    pub current_pnl: f64,
}

impl RiskManager {
    pub fn new(max_pos: Volume, max_loss: f64) -> Self {
        Self {
            max_position: max_pos,
            max_daily_loss: max_loss,
            current_position: 0.0,
            current_pnl: 0.0,
        }
    }

    pub fn check_new_order(&self, side: Side, volume: Volume) -> bool {
        if self.current_pnl <= -self.max_daily_loss {
            error!("Risk Reject: Max daily loss exceeded ({})", self.current_pnl);
            return false;
        }

        let projected_position = match side {
            Side::Bid => self.current_position + volume,
            Side::Ask => self.current_position - volume,
        };

        if projected_position.abs() > self.max_position {
            error!("Risk Reject: Max position limit exceeded (Proj: {}, Max: {})", projected_position, self.max_position);
            return false;
        }

        true
    }

    pub fn update_position(&mut self, side: Side, volume: Volume, _price: f64) {
        match side {
            Side::Bid => self.current_position += volume,
            Side::Ask => self.current_position -= volume,
        }
        info!("Position updated: {}", self.current_position);
    }
}

use features::Features;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    EnterLong,
    EnterShort,
    Exit,
    None,
}

pub fn generate_signal(features: &Features) -> SignalType {
    if features.obi_top_1 > 0.6 {
        info!("Signal: Long (OBI: {:.2})", features.obi_top_1);
        SignalType::EnterLong
    } else if features.obi_top_1 < -0.6 {
        info!("Signal: Short (OBI: {:.2})", features.obi_top_1);
        SignalType::EnterShort
    } else {
        SignalType::None
    }
}

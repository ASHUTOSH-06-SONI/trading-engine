#include "features/market_features.hpp"
std::optional<double> trading::MarketFeatures::spread(const OrderBook& book) {
    auto ask = book.best_ask_level();
    auto bid = book.best_bid_level();
    if (!ask.has_value() || !bid.has_value()) {
        return std::nullopt;
    }
    return static_cast<double>(ask->price - bid->price);
}
std::optional<double> trading::MarketFeatures::mid_price(const OrderBook& book) {
    auto ask = book.best_ask_level();
    auto bid = book.best_bid_level();
    if (!ask.has_value() || !bid.has_value()) {
        return std::nullopt;
    }
    return (static_cast<double>(ask->price)+static_cast<double>(bid->price))/2.0;
}
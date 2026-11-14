#include "orderbook/order_book.hpp"
void trading::OrderBook::apply_snapshot(const std::vector<trading::Level>& bid_levels, const std::vector<trading::Level>& asks_levels){
    // clear old state
    bids.clear();
    asks.clear();
    // copy incoming bid levels 
    for (const auto& level : bid_levels) {
        bids[level.price] =level.quantity; 
    }
    // copy incoming ask levels
    for (const auto& level : asks_levels) {
        asks[level.price] = level.quantity;
    }
}
bool trading::OrderBook::apply_update(Side side, Price price, Quantity quantity){
    if (quantity < 0) {
        return false;
    }

    if(side == Side::Bid){
        if(quantity ==0) bids.erase(price);
        else if(quantity >0) bids[price] = quantity;
    }
    else if(side == Side::Ask){
        if(quantity ==0) asks.erase(price);
        else if(quantity >0) asks[price] = quantity;
    }
    return true;
}

std::optional<trading::Price> trading::OrderBook::best_bid() const {
    if (bids.empty()) {
        return std::nullopt;
    }return bids.begin()->first;
}
std::optional<trading::Price> trading::OrderBook::best_ask() const{
    if(asks.empty()){
        return std::nullopt;
    }return asks.begin()->first;
}
bool trading::OrderBook::empty() const{
    return bids.empty() && asks.empty();
}   

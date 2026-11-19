#pragma once
#include "common/types.hpp"
#include <functional>
#include <map>
#include <optional>
#include <vector>
namespace trading {
class OrderBook {
    public:
        void apply_snapshot(const std::vector<Level>& bid_levels, const std::vector<Level>& asks_levels);// incoming snapshot ka data
        bool apply_update(Side side, Price price, Quantity quantity);
        std::optional<Price> best_bid() const;
        std::optional<Price> best_ask() const;
        std::optional<Level> best_bid_level() const;
        std::optional<Level> best_ask_level() const;
        bool empty() const;
    private:
        std::map<Price, Quantity, std::greater<Price>> bids; // internal bid map- has price and quantity 
        std::map<Price, Quantity, std::less<Price>> asks;
    };
} 
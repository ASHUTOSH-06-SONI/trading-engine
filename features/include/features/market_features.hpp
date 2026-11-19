#pragma once
#include "orderbook/order_book.hpp"
#include<optional>
namespace trading{
    class MarketFeatures{
    public:
        static std::optional<double> spread(const OrderBook &book);
        static std::optional<double> mid_price(const OrderBook &book);
    };
}
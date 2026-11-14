#pragma once
#include <cstdint>
namespace trading {
    using Price = std::int64_t;
    using Quantity = std::int64_t;
    enum class Side {
        Bid,
        Ask
    };
    struct Level {
        Price price;
        Quantity quantity;
    };
    
}   
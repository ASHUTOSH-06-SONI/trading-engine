#include "features/market_features.hpp"
#include "orderbook/order_book.hpp"
#include <cassert>
#include <cmath>
#include <iostream>

static bool approx(double a, double b) {
    return std::fabs(a - b) < 1e-9;
}

int main() {
    using trading::MarketFeatures;
    using trading::OrderBook;
    using trading::Side;

    // Test 1: spread on normal book
    OrderBook book1;
    book1.apply_snapshot({{100, 5}, {99, 10}}, {{102, 3}, {103, 8}});

    auto s1 = MarketFeatures::spread(book1);
    assert(s1.has_value());
    assert(approx(*s1, 2.0));
    std::cout << "Test 1 passed: Spread on normal book\n";

    // Test 2: mid price on normal book
    auto m1 = MarketFeatures::mid_price(book1);
    assert(m1.has_value());
    assert(approx(*m1, 101.0));
    std::cout << "Test 2 passed: Mid price on normal book\n";

    // Test 3: mid price is fractional when sum is odd
    OrderBook book2;
    book2.apply_snapshot({{100, 1}}, {{101, 1}});

    auto m2 = MarketFeatures::mid_price(book2);
    assert(m2.has_value());
    assert(approx(*m2, 100.5));
    std::cout << "Test 3 passed: Fractional mid price\n";

    // Test 4: spread on empty book
    OrderBook empty_book;
    assert(!MarketFeatures::spread(empty_book).has_value());
    std::cout << "Test 4 passed: Spread on empty book\n";

    // Test 5: mid price on empty book
    assert(!MarketFeatures::mid_price(empty_book).has_value());
    std::cout << "Test 5 passed: Mid price on empty book\n";

    // Test 6: only bids exist
    OrderBook bids_only;
    bids_only.apply_snapshot({{100, 5}}, {});

    assert(!MarketFeatures::spread(bids_only).has_value());
    assert(!MarketFeatures::mid_price(bids_only).has_value());
    std::cout << "Test 6 passed: Bids only\n";

    // Test 7: only asks exist
    OrderBook asks_only;
    asks_only.apply_snapshot({}, {{101, 5}});

    assert(!MarketFeatures::spread(asks_only).has_value());
    assert(!MarketFeatures::mid_price(asks_only).has_value());
    std::cout << "Test 7 passed: Asks only\n";

    // Test 8: features track book updates
    OrderBook live;
    live.apply_snapshot({{100, 5}}, {{104, 5}});
    assert(approx(*MarketFeatures::spread(live), 4.0));

    live.apply_update(Side::Bid, 102, 3);   // better bid
    assert(approx(*MarketFeatures::spread(live), 2.0));
    assert(approx(*MarketFeatures::mid_price(live), 103.0));

    live.apply_update(Side::Bid, 102, 0);   // remove it, back to 100
    assert(approx(*MarketFeatures::spread(live), 4.0));
    std::cout << "Test 8 passed: Features follow updates\n";

    // Test 9: spread becomes nullopt once a side is emptied
    live.apply_update(Side::Ask, 104, 0);
    assert(!MarketFeatures::spread(live).has_value());
    assert(!MarketFeatures::mid_price(live).has_value());
    std::cout << "Test 9 passed: Side emptied\n";

    // Test 10: locked book (bid == ask) gives zero spread
    OrderBook locked;
    locked.apply_snapshot({{100, 5}}, {{100, 5}});

    assert(approx(*MarketFeatures::spread(locked), 0.0));
    assert(approx(*MarketFeatures::mid_price(locked), 100.0));
    std::cout << "Test 10 passed: Locked book\n";

    std::cout << "\nAll feature tests passed!" << std::endl;
    return 0;
}
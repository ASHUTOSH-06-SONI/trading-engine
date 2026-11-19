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
        // Imbalance 1: balanced top of book -> 0
    OrderBook imb_balanced;
    imb_balanced.apply_snapshot({{100, 5}}, {{101, 5}});

    auto i1 = MarketFeatures::order_book_imbalance(imb_balanced);
    assert(i1.has_value());
    assert(approx(*i1, 0.0));
    std::cout << "Imbalance 1 passed: Balanced book\n";

    // Imbalance 2: bid heavy -> positive
    OrderBook imb_bid_heavy;
    imb_bid_heavy.apply_snapshot({{100, 9}}, {{101, 1}});

    auto i2 = MarketFeatures::order_book_imbalance(imb_bid_heavy);
    assert(i2.has_value());
    assert(approx(*i2, 0.8));   // (9-1)/(9+1)
    std::cout << "Imbalance 2 passed: Bid heavy\n";

    // Imbalance 3: ask heavy -> negative
    OrderBook imb_ask_heavy;
    imb_ask_heavy.apply_snapshot({{100, 1}}, {{101, 9}});

    auto i3 = MarketFeatures::order_book_imbalance(imb_ask_heavy);
    assert(i3.has_value());
    assert(approx(*i3, -0.8));
    std::cout << "Imbalance 3 passed: Ask heavy\n";

    // Imbalance 4: fractional result
    OrderBook imb_frac;
    imb_frac.apply_snapshot({{100, 3}}, {{101, 2}});

    auto i4 = MarketFeatures::order_book_imbalance(imb_frac);
    assert(i4.has_value());
    assert(approx(*i4, 0.2));   // (3-2)/5
    std::cout << "Imbalance 4 passed: Fractional imbalance\n";

    // Imbalance 5: empty book
    OrderBook imb_empty;
    assert(!MarketFeatures::order_book_imbalance(imb_empty).has_value());
    std::cout << "Imbalance 5 passed: Empty book\n";

    // Imbalance 6: only bids
    OrderBook imb_bids_only;
    imb_bids_only.apply_snapshot({{100, 5}}, {});
    assert(!MarketFeatures::order_book_imbalance(imb_bids_only).has_value());
    std::cout << "Imbalance 6 passed: Bids only\n";

    // Imbalance 7: only asks
    OrderBook imb_asks_only;
    imb_asks_only.apply_snapshot({}, {{101, 5}});
    assert(!MarketFeatures::order_book_imbalance(imb_asks_only).has_value());
    std::cout << "Imbalance 7 passed: Asks only\n";

    // Imbalance 8: always within [-1, 1], even with extreme sizes
    OrderBook imb_extreme;
    imb_extreme.apply_snapshot({{100, 1000000}}, {{101, 1}});

    auto i8 = MarketFeatures::order_book_imbalance(imb_extreme);
    assert(i8.has_value());
    assert(*i8 <= 1.0 && *i8 >= -1.0);
    assert(*i8 > 0.99);
    std::cout << "Imbalance 8 passed: Extreme sizes stay in [-1, 1]\n";

    // Imbalance 9: only best level counts, deeper levels ignored
    OrderBook imb_depth;
    imb_depth.apply_snapshot({{100, 5}, {99, 1000}}, {{101, 5}, {102, 1000}});

    auto i9 = MarketFeatures::order_book_imbalance(imb_depth);
    assert(i9.has_value());
    assert(approx(*i9, 0.0));
    std::cout << "Imbalance 9 passed: Deep levels ignored\n";

    // Imbalance 10: follows updates
    OrderBook imb_live;
    imb_live.apply_snapshot({{100, 5}}, {{101, 5}});

    imb_live.apply_update(Side::Bid, 100, 15);
    assert(approx(*MarketFeatures::order_book_imbalance(imb_live), 0.5));   // (15-5)/20

    imb_live.apply_update(Side::Ask, 101, 45);
    assert(approx(*MarketFeatures::order_book_imbalance(imb_live), -0.5));  // (15-45)/60

    imb_live.apply_update(Side::Ask, 101, 0);   // ask side empties
    assert(!MarketFeatures::order_book_imbalance(imb_live).has_value());
    std::cout << "Imbalance 10 passed: Follows updates\n";
    std::cout << "\nAll feature tests passed!" << std::endl;
    return 0;
}

#include "orderbook/order_book.hpp"
#include <cassert>
#include <iostream>
#include <vector>

int main() {

    // Test 1: A newly created order book should be empty
    trading::OrderBook book;

    assert(book.empty());
    std::cout << "Test 1 passed: Empty order book" << std::endl;


    // Test 2: Apply an initial snapshot
    std::vector<trading::Level> bids = {
        {100, 5},
        {99, 10},
        {98, 4}
    };

    std::vector<trading::Level> asks = {
        {101, 3},
        {102, 8},
        {103, 6}
    };

    book.apply_snapshot(bids, asks);

    assert(!book.empty());
    assert(book.best_bid().has_value());
    assert(book.best_ask().has_value());
    assert(*book.best_bid() == 100);
    assert(*book.best_ask() == 101);

    std::cout << "Test 2 passed: Initial snapshot\n";


    // Test 3: Snapshot replacement
    std::vector<trading::Level> new_bids = {
        {97, 2},
        {96, 7}
    };

    std::vector<trading::Level> new_asks = {
        {104, 4},
        {105, 9}
    };

    book.apply_snapshot(new_bids, new_asks);

    assert(*book.best_bid() == 97);
    assert(*book.best_ask() == 104);

    std::cout << "Test 3 passed: Snapshot replacement\n";


    // Test 4: Update existing bid quantity (absolute quantity)
    assert(book.apply_update(trading::Side::Bid, 97, 15));
    assert(*book.best_bid() == 97);

    std::cout << "Test 4 passed: Update existing bid\n";


    // Test 5: Insert a new, better bid
    assert(book.apply_update(trading::Side::Bid, 98, 6));
    assert(*book.best_bid() == 98);

    std::cout << "Test 5 passed: Insert new bid\n";


    // Test 6: Update existing ask quantity
    assert(book.apply_update(trading::Side::Ask, 104, 12));
    assert(*book.best_ask() == 104);

    std::cout << "Test 6 passed: Update existing ask\n";


    // Test 7: Insert a new, better ask
    assert(book.apply_update(trading::Side::Ask, 103, 4));
    assert(*book.best_ask() == 103);

    std::cout << "Test 7 passed: Insert new ask\n";


    // Test 8: Delete best bid
    assert(book.apply_update(trading::Side::Bid, 98, 0));
    assert(*book.best_bid() == 97);

    std::cout << "Test 8 passed: Delete best bid\n";


    // Test 9: Delete best ask
    assert(book.apply_update(trading::Side::Ask, 103, 0));
    assert(*book.best_ask() == 104);

    std::cout << "Test 9 passed: Delete best ask\n";


    // Test 10: Delete the current best bid
    assert(book.apply_update(trading::Side::Bid, 97, 0));
    assert(book.best_bid().has_value());
    assert(*book.best_bid() == 96);
    assert(!book.empty());  // Asks still exist

    std::cout << "Test 10 passed: Delete current best bid\n";


    // Test 11: Reject negative quantity
    assert(!book.apply_update(trading::Side::Ask, 104, -5));
    assert(*book.best_ask() == 104);  // Book remains unchanged

    std::cout << "Test 11 passed: Reject negative quantity\n";


    // Test 12: Delete current best ask
    assert(book.apply_update(trading::Side::Ask, 104, 0));
    assert(book.best_ask().has_value());
    assert(*book.best_ask() == 105);

    std::cout << "Test 12 passed: Delete current best ask\n";


    // Test 13: Delete last remaining ask
    assert(book.apply_update(trading::Side::Ask, 105, 0));

    assert(!book.best_ask().has_value());
    assert(book.best_bid().has_value());   // bids still exist
    assert(*book.best_bid() == 96);
    assert(!book.empty());                 // book not empty yet

    std::cout << "Test 13 passed: Delete last remaining ask\n";

    // Test 14: Delete last remaining bid
    assert(book.apply_update(trading::Side::Bid, 96, 0));

    assert(!book.best_bid().has_value());
    assert(book.empty());

    std::cout << "Test 14 passed: Empty entire book\n";

    // Test 15: retrieve level of best bid
    trading::OrderBook fresh;
    fresh.apply_snapshot({{97, 2}, {96, 7}}, {{104, 4}});

    auto best_bid = fresh.best_bid_level();
    assert(best_bid.has_value());
    assert(best_bid->price == 97);
    assert(best_bid->quantity == 2);
    std::cout << "Test 15 passed: Retrieve best bid level\n";
    // Test 16: retrieve level of best ask
    trading::OrderBook ask_book;
    ask_book.apply_snapshot({{97, 2}}, {{101, 3}, {102, 8}});

    auto best_ask = ask_book.best_ask_level();
    assert(best_ask.has_value());
    assert(best_ask->price == 101);
    assert(best_ask->quantity == 3);
    std::cout << "Test 16 passed: Retrieve best ask level\n";


    // Test 17: best bid level on empty book
    trading::OrderBook empty_bid_book;

    assert(!empty_bid_book.best_bid_level().has_value());
    std::cout << "Test 17 passed: Best bid level on empty book\n";


    // Test 18: best ask level on empty book
    trading::OrderBook empty_ask_book;

    assert(!empty_ask_book.best_ask_level().has_value());
    std::cout << "Test 18 passed: Best ask level on empty book\n";
    // Test 19: best ask level when only bids exist
    trading::OrderBook one_sided;
    one_sided.apply_snapshot({{100, 5}}, {});

    assert(one_sided.best_bid_level().has_value());
    assert(!one_sided.best_ask_level().has_value());
    std::cout << "Test 19 passed: One-sided book\n";
    std::cout << "\nAll tests passed!" << std::endl;

    return 0;
}

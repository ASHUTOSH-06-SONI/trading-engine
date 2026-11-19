
cmake_minimum_required(VERSION 3.20)

project(trading_engine LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_library(common INTERFACE)
target_include_directories(common INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}/common/include
)

add_library(orderbook
    orderbook/src/order_book.cpp
)

target_include_directories(orderbook PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/orderbook/include
)

target_link_libraries(orderbook PUBLIC common)

# Features module
add_library(features
    features/src/market_features.cpp
)

target_include_directories(features PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/features/include
)

target_link_libraries(features PUBLIC orderbook)

# Testing
enable_testing()

add_executable(orderbook_tests
    orderbook/tests/order_book_test.cpp
)

target_link_libraries(orderbook_tests
    PRIVATE orderbook
)

add_test(
    NAME orderbook_tests
    COMMAND orderbook_tests
)

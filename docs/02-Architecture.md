# Architecture

## Actually answering "what" are we building

C++ engine that consumes crypto market data in real time, maintains its perception of the order book, calculates the features, generates signals and "simulates" trades.

## What do I wish to do with this stuff?

I want to figure out two key aspects in trading: **Performance** and **Strategy**.

- **Performance** — Particularly speed in processing, updating the book and taking the positions.
- **Strategy** — Can order-book features like OBI and microprice predict short-term price movements to survive extrinsic factors like slippage, fees, spread, etc.?

And NO, we aren't building the next HFT, lol, not on an M1. The best I have on this is MPS.

This is more of an exploratory project for me to "unlarp" my understanding of finance and systems.

Anyways, back to the architecture.

The rough pipeline for me to go about this is:

```text
Data via WebSockets
        ↓
   Feed Adapter
        ↓
 Event Normalization
        ↓
    Order Book
        ↓
   Feature Calc
        ↓
      Signal
        ↓
   Risk Checks
        ↓
  Paper Trading
        ↓
 Positions, P&L
        ↓
    Evaluations
```

We will need things like logging, tests, and performance benchmarks.

## Let's talk more deeply about all the components

### 1. Feed Adapter

Basically connects to a crypto exchange through its WebSocket API and receives the market data messages.

**Tasks:**
- Firstly establish and maintain the connection.
- Subscribe to the required channels.
- Deal with issues like disconnects or reconnects.
- Feed mess-ups, malformed messages.
- Preserve the relevant exchange timestamps and local receive timestamps.

Moreover, the adapter should know about the "format" in which the message comes from the exchange, but the other components shouldn't be bothered about the format.

### 2. Event Normalization

Normalization here is more about converting exchange messages to a format which this engine understands.

For now, we deal with:
- Order-book snapshots
- Order-book updates
- Trades

We'll also handle missing updates so that our local order book doesn't silently become incorrect.

### 3. Order Book

- Retrieve the best bid, best ask, spread, mid-price, and available quantities.

This isn't a matching engine, BTW.

### 4. Features

Basically calculate stuff like OBI, microprice, spread, mid-price.

But more stuff will be added as there's more detailing to this architecture.

### 5. Strategy

For now, gonna start with a simple OBI-based strategy.

Buy if OBI is strongly positive and sell if strongly negative — very basic HFT strategy.

### 6. Risk and Execution

Before executing even a simulated trade, check basic limits like maximum position and order size.

The paper-trading simulator will track orders, fills, fees, positions, and P&L.

No real-money trading for now.

### 7. Logging and Benchmarks

Track errors, signals, simulated trades, and performance.

Measure how many events we process per second and how long it takes to go from receiving market data to generating a decision.

### 8. Implementation Order

- First, the exchange and the WebSocket feed. I've some idea about WebSockets coz I'd worked with them quite recently. This video's a gem for that, BTW: https://www.youtube.com/watch?v=DurNPIYPTno
- Define the basic data types and events.
- Connect to the feed and parse messages.
- Build and test the order book.
- Calculate OBI, microprice, and other basic features.
- Implement the OBI strategy.
- Add risk checks and paper trading.
- Evaluate the strategy using market data and trading costs.
- Finally, benchmark and optimize the engine.

### 9. Some Rules (Yea, Sure)

- Simplicity has to be the top priority.
- Accuracy first, only then we move to optimization, but that accuracy also gets benchmarked.
- Components are all separated, easier to debug, obv.
- Record market events so we can replay them and reproduce bugs.
- And ofc, code entirely on your own, understand where there's a flaw or a potential flaw, document every decision, every unique operator and the reasoning behind it.

Anyways, let's get to writing code.

Peace......
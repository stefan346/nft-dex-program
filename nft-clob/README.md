# NFT exchange
We're on a mission to make NFT investments more attractive for HNWI clients. 

## Tests
Run benchmarks using criterion for order book performance measurement
```
cargo bench
```

## Floor central limit order book
### Program instructions
- SwapFromNFT: Swap one or more NFTs from the same collection for fungible-tokens.
- SwapFromFT: Swap fungible-tokens for NFTs in the same collection.
- NewOrd: Place new order in the limit order book.
  - Order types
    - MO: A Maker-Only order (MO) is a GTC order except it will be rejected and cancelled if the price entered would execute immediately e.g. a buy limit order above market price, entered as a maker-only, would be rejected and cancelled.
    - IOC: An Immediate-Or-Cancel order (IOC) is a buy or sell order that attempts to execute all or part immediately and then cancels any unfilled portion of the order.
    - FOK: A Fill-Or-Kill order (FOK) is a buy or sell order that must be executed immediately in its entirety; otherwise, the entire order will be cancelled (i.e., no partial execution of the order is allowed).
    - GTC: A Good-Til-Cancelled order (GTC) is a buy or sell order that remains active until it is either executed or until the user cancels it.
  - Order state
    - Pending
    - Rejected
    - New
    - Partially filled
    - Filled
- CancelOrd: Cancel order from the limit order book.
- NewInstrmtGrp: Create a new instrument group.
- NewInstrmt: Create a new instrument and assign it to an instrument group.

### Account states
- InstrmtGrp
- Instrmt
  - Symbol
  - Market
- Order
  - CumQty: Cumulative quantity. Total number of fills
  - LeavesQty: Leaves quantity. Amount open for further execution
  - ClOrd: Client order id. An ID assigned by the client.
  - OrderId: Unique identifier for the order assigned by the program.
  - AvgPx: Average price
  - 
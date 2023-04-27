use anchor_lang::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum BookSide {
    Sell,
    Buy,
}

/// All order types must have a limit price upon placement.
#[derive(Debug, Copy, Clone, AnchorSerialize, AnchorDeserialize, Eq, PartialEq)]
pub enum OrderType {
    /// A Good-Til-Cancelled order (GTC) is a buy or sell order that remains
    /// active until it is either executed or until the user cancels it.
    GTC = 0,
    /// A Fill-Or-Kill order (FOK) is a buy or sell order that must be
    /// executed immediately in its entirety; otherwise, the entire order will
    /// be cancelled (i.e., no partial execution of the order is allowed).
    FOK = 1,
    /// An Immediate-Or-Cancel order (IOC) is a buy or sell order that attempts
    /// to execute all or part immediately and then cancels any unfilled portion
    /// of the order.
    IOC = 2,
    /// A Maker-Only order (MO) is a GTC order except it will be rejected and
    /// cancelled if the price entered would execute immediately e.g. a buy
    /// limit order above market price, entered as a maker-only, would be
    /// rejected and cancelled.
    MO = 3
}
use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Not admin.")]
    NotAdmin,
    #[msg("Order not executed immediately in its entirety.")]
    FillOrKillFailed,
    #[msg("Order executed partially or fully as taker.")]
    MakerOnlyFailed,
    #[msg("Leaves quantity for an IOC order not set to 0 after execution!")]
    ImmediateOrCancelError,
}

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
    #[msg("Nothing to crank! Try again later.")]
    RbCrankEmpty,
    #[msg("Wrong token account.")]
    WrongTokenAccount,
    #[msg("Wrong vault account.")]
    WrongVaultAccount,
    #[msg("Unauthorized order cancellation.")]
    UnauthorizedOrderCancellation,
    #[msg("Order is tombstone!")]
    TombstoneOrder,
}

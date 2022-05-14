use solana_program::{program_error::ProgramError};
use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokensellError {
    #[error("Unknown instruction")]
    UnknownInstruction,

    #[error("Failed to unpack instruction data")]
    TokenTransferFailed,

    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,

    #[error("Invalid USDT target account")]
    InvalidUSDTTargetAccount,

    #[error("Invalid account")]
    InvalidAccount,

    #[error("Invalid account owner")]
    InvalidOwner,

    #[error("No tokens in vault")]
    NoTokensInVault,

    #[error("Token account mint mismatch")]
    WrongMint,

    #[error("Integer overflow")]
    Overflow,

    #[error("No tokens to claim")]
    NothingToClaim,

    #[error("Account is already inited")]
    AccountInitialized,

    #[error("Sell has not started yet")]
    SellNotStarted,

    #[error("Sell is already finished")]
    SellEnded,

    #[error("Invalid end timestamp")]
    InvalidEndTimestamp,

    #[error("Participants signature required")]
    SignatureRequired,

    #[error("Initial deposit cant be less than required")]
    MinimalDeposit,

    #[error("Account size mismatch")]
    SizeMismatch
}

impl From<TokensellError> for ProgramError {
    fn from(e: TokensellError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

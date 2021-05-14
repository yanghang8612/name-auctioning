use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum NameAuctionError {
    #[error("An auction for this token is still in progress")]
    AuctionInProgress,
    #[error("The bid price is too low")]
    BidTooLow,
}

impl From<NameAuctionError> for ProgramError {
    fn from(e: NameAuctionError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for NameAuctionError {
    fn type_of() -> &'static str {
        "TokenizationError"
    }
}

use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;
use num_derive::FromPrimitive;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokenizationError {
    #[error("An auction for this token is still in progress")]
    AuctionInProgress

}

impl From<TokenizationError> for ProgramError {
    fn from(e: TokenizationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenizationError {
    fn type_of() -> &'static str {
        "TokenizationError"
    }
}
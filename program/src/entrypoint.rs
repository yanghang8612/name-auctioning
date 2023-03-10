use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, decode_error::DecodeError, entrypoint, entrypoint::ProgramResult,
    msg, program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::NameAuctionError, processor::Processor};

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        error.print::<NameAuctionError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for NameAuctionError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            NameAuctionError::AuctionInProgress => msg!("Error: the auction is still in progress"),
            NameAuctionError::BidTooLow => msg!("Error: The bid price is too low"),
            NameAuctionError::AuctionRealized => {
                msg!("Error: The auction has already been run successfully")
            }
        }
    }
}

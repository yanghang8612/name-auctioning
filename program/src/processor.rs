use crate::{
    instructions::ProgramInstruction,
    processor::{claim::process_claim, create::process_create, init::process_init},
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod claim;
pub mod create;
pub mod init;

pub const OVERTIME_LENGTH: u64 = 900;
pub const PRICE_INCREMENT_MARGIN: u64 = 429496729; // 1% bid increment
pub const END_AUCTION_GAP: u64 = 10;
pub const TOKEN_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC mint
pub const MINIMUM_PRICE: u64 = 20_000_000;
pub const AUCTION_PROGRAM_ID: &str = "AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT";
pub const AUCTION_MAX_LENGTH: u64 = 604800; // One minute    in seconds
pub const BONFIDA_VAULT: &str = "DmSyHDSM9eSLyvoLsPvDr5fRRFZ7Bfr3h3ULvWpgQaq7";
pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = ProgramInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::Init { state_nonce } => {
                msg!("Instruction: Init");
                process_init(program_id, accounts, state_nonce)?;
            }
            ProgramInstruction::Create { name } => {
                msg!("Instruction: Create");
                process_create(program_id, accounts, name)?;
            }
            ProgramInstruction::Claim {
                hashed_name,
                lamports,
                space,
            } => {
                msg!("Instruction: Claim");
                process_claim(
                    program_id,
                    accounts,
                    Vec::from(hashed_name),
                    lamports,
                    space,
                )?;
            }
        }
        Ok(())
    }
}

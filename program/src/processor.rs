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

pub const OVERTIME_LENGTH: u64 = 600;
pub const PRICE_INCREMENT_MARGIN: u64 = 429496729; // 1% bid increment
pub const END_AUCTION_GAP: u64 = 3600;
pub const TOKEN_MINT: &str = "2gsgrFTjFsckJiPifzkHP3tznMuqVbh5TTUcVd3iMVQx"; // USDC mint
pub const MINIMUM_PRICE: u64 = 1_000_000;
pub const AUCTION_MAX_LENGTH: u64 = 604_800; // One week in seconds
pub const BONFIDA_VAULT: &str = "BJa7dq3bRP216zaTdw4cdcV71WkPc1HXvmnGeFVDi5DC";
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
            ProgramInstruction::Create { hashed_name } => {
                msg!("Instruction: Create");
                process_create(program_id, accounts, Vec::from(hashed_name))?;
            }
            ProgramInstruction::Claim {
                hashed_name,
                lamports,
                space,
            } => {
                msg!("Instruction: Claim");
                process_claim(program_id, accounts, hashed_name, lamports, space)?;
            }
        }
        Ok(())
    }
}

use crate::{
    instructions::ProgramInstruction,
    processor::{
        claim::process_claim, create::process_create, create_reverse::process_create_reverse,
        init::process_init, resell::process_resell, reset_auction::process_reset_auction,
    },
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod claim;
pub mod create;
pub mod create_reverse;
pub mod init;
pub mod resell;
pub mod reset_auction;

////////////////////////////////////////////////////////////

pub const OVERTIME_LENGTH: u64 = 900;
pub const PRICE_INCREMENT_MARGIN: u64 = 429496729; // 1% bid increment
pub const END_AUCTION_GAP: u64 = 600;
pub const TOKEN_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC mint
pub const MINIMUM_PRICE: u64 = 20_000_000;
pub const AUCTION_PROGRAM_ID: &str = "AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT";
pub const BONFIDA_USDC_VAULT: &str = "DmSyHDSM9eSLyvoLsPvDr5fRRFZ7Bfr3h3ULvWpgQaq7";
pub const AUCTION_MAX_LENGTH: u64 = 604800; // One week in seconds
pub const ADMIN: &str = "BD4vT1aztHmuEPZh7GgvpeFskgyhi9AtPwtxzYEh5J91";
pub const FEES: &[u64] = &[500, 300, 200, 150, 100]; // Fees for low leverage orders for tiers [0, 1 ,2]
pub const FEE_TIERS: [u64; 4] = [10_000_000, 100_000_000, 500_000_000, 1_000_000_000]; // Amount of FIDA tokens (with precision) that the discount account needs to hold
pub const FIDA_MINT: &str = "EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp";
pub const ROOT_DOMAIN_ACCOUNT: &str = "58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx";
pub const BONFIDA_SOL_VAULT: &str = "GcWEQ9K78FV7LEHteFVciYApERk5YvQuFDQPk1yYJVXi";

// Fees taken for the reselling of domain names
// | Tier | Percentage of payout    | Requirements   |
// | ---- | ----------------------- | -------------- |
// | 0    | 5%                      | None           |
// | 1    | 3%                      | 10 FIDA        |
// | 2    | 2%                      | 100 FIDA       |
// | 3    | 1.5%                    | 500 FIDA       |
// | 4    | 1%                      | 1,000 FIDA     |

////////////////////////////////////////////////////////////

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
            ProgramInstruction::Create { name, max_price } => {
                msg!("Instruction: Create");
                process_create(program_id, accounts, name, max_price)?;
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
            ProgramInstruction::ResetAuction => {
                msg!("Instruction: Reset auction (admin command)");
                process_reset_auction(program_id, accounts)?;
            }
            ProgramInstruction::Resell {
                name,
                minimum_price,
                end_auction_at,
                max_price,
            } => {
                msg!("Instruction: Resell");
                process_resell(
                    program_id,
                    accounts,
                    name,
                    minimum_price,
                    end_auction_at,
                    max_price,
                )?;
            }
            ProgramInstruction::CreateReverse { name } => {
                msg!("Instruction: Create Reverse");
                process_create_reverse(program_id, accounts, name)?;
            }
        }
        Ok(())
    }
}

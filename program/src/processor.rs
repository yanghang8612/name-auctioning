use crate::{
    instructions::ProgramInstruction,
    processor::{
        admin_claim::process_a_claim, claim::process_claim, create::process_create,
        create_admin::process_create_admin, create_reverse::process_create_reverse,
        create_v2::process_create_v2, end_auction::process_end_auction, init::process_init,
        resell::process_resell, reset_auction::process_reset_auction,
    },
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey,
    pubkey::Pubkey,
};

pub mod admin_claim;
pub mod claim;
pub mod create;
pub mod create_admin;
pub mod create_reverse;
pub mod create_v2;
pub mod end_auction;
pub mod init;
pub mod resell;
pub mod reset_auction;

////////////////////////////////////////////////////////////

pub const OVERTIME_LENGTH: u64 = 900;
pub const PRICE_INCREMENT_MARGIN: u64 = 429496729; // 1% bid increment
pub const END_AUCTION_GAP: u64 = 600;
pub const TOKEN_MINT: Pubkey = pubkey!("EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp"); // FIDA mint
pub const MINIMUM_PRICE_USD: u64 = 20_000_000; // 20 USD
pub const AUCTION_PROGRAM_ID: Pubkey = pubkey!("AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT");
pub const BONFIDA_FIDA_VAULT: Pubkey = pubkey!("AUoZ3YAhV3b2rZeEH93UMZHXUZcTramBvb4d9YEVySkc");
pub const AUCTION_MAX_LENGTH: u64 = 259200; // 3 days in seconds
pub const ADMIN: Pubkey = pubkey!("BD4vT1aztHmuEPZh7GgvpeFskgyhi9AtPwtxzYEh5J91");
pub const FEES: &[u64] = &[500, 300, 200, 150, 100]; // Fees for low leverage orders for tiers [0, 1 ,2]
pub const FEE_TIERS: [u64; 4] = [10_000_000, 100_000_000, 500_000_000, 1_000_000_000]; // Amount of FIDA tokens (with precision) that the discount account needs to hold
pub const FIDA_MINT: Pubkey = pubkey!("EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp");
pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx");
pub const BONFIDA_SOL_VAULT: Pubkey = pubkey!("GcWEQ9K78FV7LEHteFVciYApERk5YvQuFDQPk1yYJVXi");
pub const ADMIN_CREATE_KEY: Pubkey = pubkey!("CHG6XM8Ugk7xkZMcp5PMoJTYfwCG7XStv5rt2w1eQKPS");
pub const BONFIDA_USDC_VAULT: Pubkey = pubkey!("DmSyHDSM9eSLyvoLsPvDr5fRRFZ7Bfr3h3ULvWpgQaq7");
pub const ADMIN_CLAIM_KEY: Pubkey = pubkey!("VBx642K1hYGLU5Zm1CHW1uRXAtFgxN5mRqyMcXnLZFW");
pub const PYTH_FIDA_PRICE_ACC: Pubkey = pubkey!("ETp9eKXVv1dWwHSpsXRUuXHmw24PwRkttCGVgpZEY9zF");

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

            ProgramInstruction::CreateAdmin(params) => {
                msg!("Instruction: Create admin");
                process_create_admin(program_id, accounts, params)?;
            }
            ProgramInstruction::ClaimAdmin {
                hashed_name,
                lamports,
                space,
            } => {
                msg!("Instruction: A Claim");
                process_a_claim(program_id, accounts, hashed_name.to_vec(), lamports, space)?;
            }
            ProgramInstruction::EndAuction { name } => {
                msg!("Instruction: End Auction");
                process_end_auction(program_id, accounts, name)?;
            }
            ProgramInstruction::CreateV2 { name, space } => {
                msg!("Instruction: Create v2");
                process_create_v2(program_id, accounts, name, space)?;
            }
        }
        Ok(())
    }
}

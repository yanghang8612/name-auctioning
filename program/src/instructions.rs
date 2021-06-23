use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::processor::BONFIDA_VAULT;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum ProgramInstruction {
    /// Creates an auction
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The system program account
    ///   3. `[]` The SPL token program account
    ///   4. `[writable]` The new signer account
    ///   5. `[signer]` The parent authority account
    ///   6. `[signer, writable]` The fee payer account
    ///   7. `[writable]` The new nft token mint account
    ///   8. `[writable]` The nft token account to be created and minted to
    ///   9. `[]` The owner of the target nft token account
    Init { state_nonce: u8 },
    /// Creates an auction
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The system program account
    ///   3. `[]` The SPL token program account
    ///   4. `[writable]` The new signer account
    ///   5. `[signer]` The parent authority account
    ///   6. `[signer, writable]` The fee payer account
    ///   7. `[writable]` The new nft token mint account
    ///   8. `[writable]` The nft token account to be created and minted to
    ///   9. `[]` The owner of the target nft token account
    Create { name: String },
    /// Executes an arbitrary program instruction, signing as the tokenized authority
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The nft token account
    ///   2. `[signer]` The owner of the nft token account
    ///   3. `[]` The associateed signer account
    ///   3. `[]` The target program account
    ///   4... `[?]` The necessary accounts for the instruction, in instruction order.
    ///               All instances of the signer account will be set as signer when calling the instruction
    Claim {
        hashed_name: [u8; 32],
        lamports: u64,
        space: u32,
    },
}

pub fn init(
    program_id: Pubkey,
    state_account: Pubkey,
    fee_payer: Pubkey,
    state_nonce: u8,
) -> Instruction {
    let data = ProgramInstruction::Init { state_nonce }
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new(state_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new(fee_payer, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    program_id: Pubkey,
    auction_program_id: Pubkey,
    root_domain: Pubkey,
    name_account: Pubkey,
    reverse_lookup_account: Pubkey,
    auction_account: Pubkey,
    central_state_account: Pubkey,
    state_account: Pubkey,
    fee_payer: Pubkey,
    quote_mint: Pubkey,
    name: String,
) -> Instruction {
    let data = ProgramInstruction::Create { name }.try_to_vec().unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new_readonly(name_account, false),
        AccountMeta::new(reverse_lookup_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(auction_program_id, false),
        AccountMeta::new(auction_account, false),
        AccountMeta::new(central_state_account, false),
        AccountMeta::new(state_account, false),
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(quote_mint, false),
    ];
    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn claim(
    program_id: Pubkey,
    auction_program_id: Pubkey,
    root_domain: Pubkey,
    name_account: Pubkey,
    auction_account: Pubkey,
    state_account: Pubkey,
    central_state_account: Pubkey,
    fee_payer: Pubkey,
    quote_mint: Pubkey,
    bidder_wallet: Pubkey,
    bidder_pot: Pubkey,
    bidder_pot_token: Pubkey,
    lamports: u64,
    space: u32,
    hashed_name: [u8; 32],
) -> Instruction {
    let data = ProgramInstruction::Claim {
        hashed_name,
        lamports,
        space,
    }
    .try_to_vec()
    .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new_readonly(name_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new(auction_account, false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new(state_account, false),
        AccountMeta::new_readonly(auction_program_id, false),
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(quote_mint, false),
        AccountMeta::new(Pubkey::from_str(BONFIDA_VAULT).unwrap(), false),
        AccountMeta::new_readonly(bidder_wallet, false),
        AccountMeta::new(bidder_pot, false),
        AccountMeta::new(bidder_pot_token, false),
    ];
    Instruction {
        program_id,
        accounts,
        data,
    }
}

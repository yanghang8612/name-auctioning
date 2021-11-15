use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::processor::BONFIDA_FIDA_VAULT;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum ProgramInstruction {
    /// Ihnitiates the central state of the auction program
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[writable]` The central state account
    ///   2. `[]` The system program account
    ///   3. `[writable, signer]` The fee payer account
    ///   4. `[]` The sysvar rent account
    Init {
        state_nonce: u8,
    },
    /// Creates an auction
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The clock sysvar account
    ///   3. `[]` The name service program account
    ///   4. `[]` The root domain account
    ///   5. `[]` The name account
    ///   6. `[writable]` The reverse lookup account
    ///   7. `[]` The system program account
    ///   8. `[]` The auction program account
    ///   9. `[writable]` The auction account
    ///   10. `[]` The central state account
    ///   11. `[writable]` The state account
    ///   12. `[writable, signer]` The fee payer account
    ///   13. `[writable]` The quote mint account
    ///   14. `[writable]` The buy now account
    Create {
        name: String,
    },
    /// Executes an arbitrary program instruction, signing as the tokenized authority
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The sysvar clock account
    ///   2. `[]` The spl token program
    ///   3. `[]` The spl name service program
    ///   4. `[]` The root domain account
    ///   5. `[]` The name account
    ///   6. `[]` The system program
    ///   7. `[]` The auction program
    ///   8. `[writable]` The auction account
    ///   9. `[]` The central state account
    ///   10. `[writable]` The state account
    ///   11. `[writable]` The reselling account
    ///   12. `[writable, signer]` The fee payer account
    ///   13. `[writable]` The quote mint account
    ///   14. `[writable]` The payout destination token account
    ///   15. `[signer]` The bidder wallet account
    ///   16. `[writable]` The bidder pot account
    ///   17. `[writable]` The bidder pot token account
    ///   18. `[]` The bonfida vault account
    ///   19. `[]` The fida discount account
    ///   20. `[writable]` The buy now account
    ///   21. `[writable]` The Bonfida SOL vault account
    Claim {
        hashed_name: [u8; 32],
        lamports: u64,
        space: u32,
    },
    ResetAuction,
    /// Creates a secondary auction for domain owners to resell their ownership
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The clock sysvar account
    ///   3. `[]` The name service program account
    ///   4. `[]` The root domain account
    ///   5. `[writable]` The name account
    ///   6. `[signer]` The name owner account
    ///   7. `[writable]` The reverse lookup account
    ///   8. `[]` The system program account
    ///   9. `[]` The auction program account
    ///   10. `[writable]` The auction account
    ///   11. `[]` The central state account
    ///   12. `[writable]` The state account
    ///   13. `[writable]` The reselling state account
    ///   14. `[writable]` The destination token account
    ///   15. `[writable, signer]` The fee payer account
    ///   16. `[writable]` The buy now account
    Resell {
        name: String,
        minimum_price: u64,
        end_auction_at: u64, // Unix timestamp
        max_price: Option<u64>,
    },
    /// Creates a reverse lookup name registry for a domain name
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The clock sysvar account
    ///   3. `[]` The name service program account
    ///   4. `[]` The root domain account
    ///   6. `[writable]` The reverse lookup account
    ///   7. `[]` The system program account
    ///   10. `[]` The central state account
    ///   12. `[writable, signer]` The fee payer account
    CreateReverse {
        name: String,
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
        AccountMeta::new_readonly(central_state_account, false),
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
pub fn create_reverse(
    program_id: Pubkey,
    root_domain: Pubkey,
    reverse_lookup_account: Pubkey,
    central_state_account: Pubkey,
    fee_payer: Pubkey,
    name: String,
) -> Instruction {
    let data = ProgramInstruction::CreateReverse { name }
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new(reverse_lookup_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new(fee_payer, true),
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
    destination_token_account: Pubkey,
    quote_mint: Pubkey,
    bidder_wallet: Pubkey,
    bidder_pot: Pubkey,
    bidder_pot_token: Pubkey,
    lamports: u64,
    space: u32,
    hashed_name: [u8; 32],
    discount_account: Pubkey,
    buy_now: Pubkey,
    bonfida_sol_vault: Pubkey,
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
        AccountMeta::new(destination_token_account, false),
        AccountMeta::new_readonly(bidder_wallet, true),
        AccountMeta::new(bidder_pot, false),
        AccountMeta::new(bidder_pot_token, false),
        AccountMeta::new(Pubkey::from_str(BONFIDA_FIDA_VAULT).unwrap(), false),
        AccountMeta::new_readonly(discount_account, false),
        AccountMeta::new(buy_now, false),
        AccountMeta::new(bonfida_sol_vault, false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn resell(
    program_id: Pubkey,
    auction_program_id: Pubkey,
    root_domain: Pubkey,
    name_account: Pubkey,
    name_owner_account: Pubkey,
    reverse_lookup_account: Pubkey,
    auction_account: Pubkey,
    central_state_account: Pubkey,
    state_account: Pubkey,
    fee_payer: Pubkey,
    reselling_state_account: Pubkey,
    destination_token_account: Pubkey,
    name: String,
    minimum_price: u64,
    auction_duration: u64,
    max_price: Option<u64>,
) -> Instruction {
    let data = ProgramInstruction::Resell {
        name,
        minimum_price,
        end_auction_at: auction_duration,
        max_price,
    }
    .try_to_vec()
    .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new(name_account, false),
        AccountMeta::new_readonly(name_owner_account, true),
        AccountMeta::new(reverse_lookup_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(auction_program_id, false),
        AccountMeta::new(auction_account, false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new(state_account, false),
        AccountMeta::new(reselling_state_account, false),
        AccountMeta::new(destination_token_account, false),
        AccountMeta::new(fee_payer, true),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}
pub fn reset_auction(
    program_id: Pubkey,
    auction_program_id: Pubkey,
    admin: Pubkey,
    auction: Pubkey,
    name: Pubkey,
    state: Pubkey,
) -> Instruction {
    let data = ProgramInstruction::ResetAuction.try_to_vec().unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(auction_program_id, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(admin, true),
        AccountMeta::new(auction, false),
        AccountMeta::new_readonly(name, false),
        AccountMeta::new_readonly(state, false),
    ];
    Instruction {
        program_id,
        accounts,
        data,
    }
}

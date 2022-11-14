use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

pub use crate::processor::create_admin;
use crate::processor::{
    BONFIDA_FIDA_VAULT, BONFIDA_SOL_VAULT, BONFIDA_USDC_VAULT, CENTRAL_STATE, PYTH_FIDA_PRICE_ACC,
    ROOT_DOMAIN_ACCOUNT, USDC_MINT,
};

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
    ///   15. `[]` The Pyth Fida price account
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
    //
    // Admin instruction used only to reserve domains for ecosystem projects and prevent squatting
    //
    // Accounts expected by this instruction
    //
    // | Index | Writable | Signer | Description                      |
    // |-------|----------|--------|----------------------------------|
    // | 0     | ✅        | ✅      | The fee payer account            |
    // | 1     | ✅        | ✅      | The admin account                |
    // | 2     | ❌        | ❌      | The name service program account |
    // | 3     | ❌        | ❌      | The root domain account          |
    // | 4     | ✅        | ❌      | The name account                 |
    // | 5     | ✅        | ❌      | The reverse lookup account       |
    // | 6     | ❌        | ❌      | The central state account        |
    // | 7     | ❌        | ❌      | The system program account       |
    // | 8     | ❌        | ❌      | The rent sysvar account          |
    CreateAdmin(create_admin::Params),
    //
    // Admin instruction used to force the claim of a broken name
    //
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
    ///   21. `[signer]` The claim admin
    ClaimAdmin {
        hashed_name: [u8; 32],
        space: u32,
    },
    /// End a reselling auction
    ///
    /// Accounts expected by this instruction:
    ///
    EndAuction {
        name: String,
    },

    /// Create v2
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                   |
    /// |-------|----------|--------|-------------------------------|
    /// | 0     | ❌        | ❌      | The rent sysvar account       |
    /// | 1     | ❌        | ❌      | The naming service program ID |
    /// | 2     | ❌        | ❌      | The root domain account       |
    /// | 3     | ✅        | ❌      | The name account              |
    /// | 4     | ✅        | ❌      | The reverse look up account   |
    /// | 5     | ❌        | ❌      | The system program account    |
    /// | 6     | ❌        | ❌      | The central state account     |
    /// | 7     | ✅        | ✅      | The buyer account             |
    /// | 8     | ✅        | ❌      | The buyer token account       |
    /// | 9     | ❌        | ❌      | The quote mint account        |
    /// | 10    | ❌        | ❌      | The Pyth FIDA price account   |
    /// | 11    | ✅        | ❌      | The FIDA vault account        |
    /// | 12    | ❌        | ❌      | The SPL token program         |
    CreateV2 {
        name: String,
        space: u32,
    },
    /// Take back a domain name
    /// Accounts expected by this instruction
    ///
    /// | Index | Writable | Signer | Description                   |
    /// |-------|----------|--------|-------------------------------|
    /// | 0     | ✅        | ✅      | The admin account             |
    /// | 1     | ❌        | ❌      | The name service program id   |
    /// | 2     | ✅        | ❌      | The name account              |
    /// | 3     | ❌        | ❌      | The central state account     |
    /// | 4     | ❌        | ❌      | The class (Pubkey::default()) |
    /// | 5     | ❌        | ❌      | The .sol TLD                  |
    TakeBack,
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
        AccountMeta::new_readonly(PYTH_FIDA_PRICE_ACC, false),
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
    parent_name_opt: Option<Pubkey>,
    parent_name_owner_opt: Option<Pubkey>,
) -> Instruction {
    let data = ProgramInstruction::CreateReverse { name }
        .try_to_vec()
        .unwrap();
    let mut accounts = vec![
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new(reverse_lookup_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new(fee_payer, true),
    ];
    if let Some(k) = parent_name_opt {
        accounts.push(AccountMeta::new(k, false));
        accounts.push(AccountMeta::new_readonly(
            parent_name_owner_opt.unwrap(),
            true,
        ));
    }
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
    space: u32,
    hashed_name: [u8; 32],
    discount_account: Pubkey,
    buy_now: Pubkey,
    bonfida_sol_vault: Pubkey,
) -> Instruction {
    let data = ProgramInstruction::Claim { hashed_name, space }
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
        AccountMeta::new(BONFIDA_FIDA_VAULT, false),
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

#[allow(clippy::too_many_arguments)]
pub fn create_admin(
    program_id: Pubkey,
    fee_payer: Pubkey,
    admin: Pubkey,
    root_domain: Pubkey,
    name_account: Pubkey,
    reverse_lookup_account: Pubkey,
    central_state_account: Pubkey,
    params: create_admin::Params,
) -> Instruction {
    let data = ProgramInstruction::CreateAdmin(params)
        .try_to_vec()
        .unwrap();

    let accounts = vec![
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(admin, true),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new(name_account, false),
        AccountMeta::new(reverse_lookup_account, false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn admin_claim(
    program_id: Pubkey,
    auction_program_id: Pubkey,
    name_account: Pubkey,
    auction_account: Pubkey,
    state_account: Pubkey,
    central_state_account: Pubkey,
    bidder_wallet: Pubkey,
    bidder_pot: Pubkey,
    bidder_pot_token: Pubkey,
    admin: Pubkey,
    new_name_owner: Pubkey,
    fee_payer: Pubkey,
    root_name: Pubkey,
    hashed_name: [u8; 32],
    space: u32,
) -> Instruction {
    let data = ProgramInstruction::ClaimAdmin { hashed_name, space }
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new(name_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(auction_program_id, false),
        AccountMeta::new(auction_account, false),
        AccountMeta::new_readonly(central_state_account, false),
        AccountMeta::new(state_account, false),
        AccountMeta::new_readonly(bidder_wallet, false),
        AccountMeta::new(bidder_pot, false),
        AccountMeta::new(bidder_pot_token, false),
        AccountMeta::new(BONFIDA_USDC_VAULT, false),
        AccountMeta::new_readonly(admin, true),
        AccountMeta::new_readonly(new_name_owner, false),
        AccountMeta::new(fee_payer, true),
        AccountMeta::new_readonly(root_name, false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn end_auction(
    program_id: Pubkey,
    root_domain: Pubkey,
    name_account: Pubkey,
    auction: Pubkey,
    central_state: Pubkey,
    state: Pubkey,
    auction_program_id: Pubkey,
    auction_creator: Pubkey,
    reselling_state: Pubkey,
    destination_token: Pubkey,
    name: String,
) -> Instruction {
    let data = ProgramInstruction::EndAuction { name }
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new(name_account, false),
        AccountMeta::new_readonly(auction_program_id, false),
        AccountMeta::new(auction, false),
        AccountMeta::new_readonly(central_state, false),
        AccountMeta::new(state, false),
        AccountMeta::new(auction_creator, true),
        AccountMeta::new(reselling_state, false),
        AccountMeta::new(destination_token, false),
        AccountMeta::new(BONFIDA_SOL_VAULT, false),
        AccountMeta::new(system_program::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_v2(
    program_id: Pubkey,
    root_domain: Pubkey,
    name_account: Pubkey,
    reverse_lookup: Pubkey,
    central_state: Pubkey,
    buyer: Pubkey,
    buyer_token_source: Pubkey,
    state: Pubkey,
    name: String,
    space: u32,
) -> Instruction {
    let data = ProgramInstruction::CreateV2 { name, space }
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new_readonly(root_domain, false),
        AccountMeta::new(name_account, false),
        AccountMeta::new(reverse_lookup, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(central_state, false),
        AccountMeta::new(buyer, true),
        AccountMeta::new(buyer_token_source, false),
        AccountMeta::new(BONFIDA_USDC_VAULT, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(state, false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn take_back(
    program_id: Pubkey,
    admin: Pubkey,
    name_account: Pubkey,
    new_owner: Pubkey,
) -> Instruction {
    let data = ProgramInstruction::TakeBack.try_to_vec().unwrap();

    let accounts = vec![
        AccountMeta::new(admin, true),
        AccountMeta::new_readonly(spl_name_service::id(), false),
        AccountMeta::new(name_account, false),
        AccountMeta::new_readonly(CENTRAL_STATE, false),
        AccountMeta::new_readonly(Pubkey::default(), false),
        AccountMeta::new_readonly(ROOT_DOMAIN_ACCOUNT, false),
        AccountMeta::new_readonly(new_owner, false),
    ];

    Instruction {
        program_id,
        accounts,
        data,
    }
}

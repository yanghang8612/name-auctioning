use std::str::FromStr;

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
    sysvar::{self, Sysvar},
};
use spl_name_service::state::get_seeds_and_key;

use crate::{
    state::NameAuction,
    utils::{check_account_key, check_account_owner, check_signer, Cpi},
};

use super::{AUCTION_MAX_LENGTH, AUCTION_PROGRAM_ID};

struct Accounts<'a, 'b: 'a> {
    rent_sysvar: &'a AccountInfo<'b>,
    clock_sysvar: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
    quote_mint: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        rent_sysvar: next_account_info(accounts_iter)?,
        clock_sysvar: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        auction_program: next_account_info(accounts_iter)?,
        auction: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
        quote_mint: next_account_info(accounts_iter)?,
    };

    check_account_key(a.rent_sysvar, &sysvar::rent::id()).unwrap();
    check_account_key(a.clock_sysvar, &sysvar::clock::id()).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_key(
        a.auction_program,
        &Pubkey::from_str(AUCTION_PROGRAM_ID).unwrap(),
    )
    .unwrap();
    // check_account_owner(a.auction, &spl_auction::id()).unwrap();
    check_account_owner(a.state, &system_program::id()).unwrap();
    check_signer(a.fee_payer).unwrap();

    Ok(a)
}

pub fn process_create(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    hashed_name: Vec<u8>,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    if hashed_name.len() != 32 {
        msg!("Invalid seed length");
        return Err(ProgramError::InvalidArgument);
    }

    let (name_account_key, key) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_name,
        None,
        Some(accounts.root_domain.key),
    );

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }

    if accounts.name.data_len() != 0 {
        msg!("Name account is already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if accounts.state.data_len() != 0 {
        msg!("An auction for this name has already been created.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let signer_seeds = name_account_key.to_bytes();

    let (derived_state_key, derived_signer_nonce) =
        Pubkey::find_program_address(&[&signer_seeds], program_id);

    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let signer_seeds: &[&[u8]] = &[&signer_seeds, &[derived_signer_nonce]];

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.state,
        accounts.rent_sysvar,
        signer_seeds,
        NameAuction::LEN,
    )?;

    let current_timestamp = Clock::from_account_info(accounts.clock_sysvar)?.unix_timestamp as u64;
    let end_auction_at = Some(current_timestamp + AUCTION_MAX_LENGTH);

    let state = NameAuction {
        is_initialized: true,
        quote_mint: accounts.quote_mint.key.to_bytes(),
        signer_nonce: derived_signer_nonce,
        auction_account: accounts.auction.key.to_bytes(),
    };

    {
        let mut pt: &mut [u8] = &mut accounts.state.data.borrow_mut();
        state.serialize(&mut pt)?;
    }

    msg!("Setting up auction");

    Cpi::create_auction(
        accounts.auction_program,
        accounts.rent_sysvar,
        accounts.system_program,
        accounts.auction,
        accounts.fee_payer,
        end_auction_at,
        accounts.state,
        *accounts.name.key,
        &signer_seeds,
    )?;

    msg!("Starting auction");

    Cpi::start_auction(
        accounts.auction_program,
        accounts.clock_sysvar,
        accounts.auction,
        accounts.state,
        *accounts.name.key,
        &signer_seeds,
    )?;

    Ok(())
}

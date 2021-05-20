use std::str::FromStr;

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::hashv,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program, sysvar,
};
use spl_name_service::state::{get_seeds_and_key, HASH_PREFIX};

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
    reverse_lookup: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
    quote_mint: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        rent_sysvar: next_account_info(accounts_iter)?,
        clock_sysvar: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        reverse_lookup: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        auction_program: next_account_info(accounts_iter)?,
        auction: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
        quote_mint: next_account_info(accounts_iter)?,
    };

    check_account_key(a.rent_sysvar, &sysvar::rent::id()).unwrap();
    check_account_key(a.clock_sysvar, &sysvar::clock::id()).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_owner(a.central_state, &program_id).unwrap();
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
    name: String,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &name).as_bytes()])
        .0
        .to_vec();

    if hashed_name.len() != 32 {
        msg!("Invalid seed length");
        return Err(ProgramError::InvalidArgument);
    }

    let (name_account_key, _) = get_seeds_and_key(
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

    let hashed_reverse_lookup =
        hashv(&[(HASH_PREFIX.to_owned() + &name_account_key.to_string()).as_bytes()])
            .0
            .to_vec();

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(accounts.central_state.key),
        None,
    );

    if &reverse_lookup_account_key != accounts.reverse_lookup.key {
        msg!("Provided wrong reverse lookup account");
        return Err(ProgramError::InvalidArgument);
    }

    let signer_seeds = name_account_key.to_bytes();

    let (derived_state_key, derived_signer_nonce) =
        Pubkey::find_program_address(&[&signer_seeds], program_id);

    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let signer_seeds: &[&[u8]] = &[&signer_seeds, &[derived_signer_nonce]];

    let central_state_nonce = accounts.central_state.data.borrow()[0];

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.state,
        accounts.rent_sysvar,
        signer_seeds,
        NameAuction::LEN,
    )?;
    let end_auction_at = Some(AUCTION_MAX_LENGTH);

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

    Cpi::create_reverse_lookup_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.reverse_lookup,
        accounts.fee_payer,
        name,
        hashed_reverse_lookup,
        accounts.central_state,
        accounts.rent_sysvar,
        central_state_signer_seeds,
    )?;

    Ok(())
}

use std::str::FromStr;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
    sysvar::{self},
};
use spl_name_service::state::get_seeds_and_key;

use crate::{
    state::NameAuction,
    utils::{check_account_key, check_account_owner, check_signer, Cpi},
};

use super::BONFIDA_VAULT;

struct Accounts<'a, 'b: 'a> {
    clock_sysvar: &'a AccountInfo<'b>,
    spl_token_program: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
    quote_mint: &'a AccountInfo<'b>,
    bonfida_vault: &'a AccountInfo<'b>,
    bidder_wallet: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    bidder_pot_token: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        clock_sysvar: next_account_info(accounts_iter)?,
        spl_token_program: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        auction_program: next_account_info(accounts_iter)?,
        auction: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
        quote_mint: next_account_info(accounts_iter)?,
        bonfida_vault: next_account_info(accounts_iter)?,
        bidder_wallet: next_account_info(accounts_iter)?,
        bidder_pot: next_account_info(accounts_iter)?,
        bidder_pot_token: next_account_info(accounts_iter)?,
    };
    check_account_key(a.clock_sysvar, &sysvar::clock::id()).unwrap();
    check_account_key(a.spl_token_program, &spl_token::id()).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_owner(a.name, &spl_name_service::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_key(a.auction_program, &spl_auction::id()).unwrap();
    check_account_owner(a.auction, &spl_auction::id()).unwrap();
    check_account_owner(a.central_state, &program_id).unwrap();
    check_account_owner(a.state, &program_id).unwrap();
    check_account_key(a.bonfida_vault, &Pubkey::from_str(BONFIDA_VAULT).unwrap()).unwrap();
    check_signer(a.bidder_wallet).unwrap();

    Ok(a)
}

pub fn process_claim(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    hashed_name: Vec<u8>,
    lamports: u64,
    space: u32,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    if hashed_name.len() != 32 {
        msg!("Invalid seed length");
        return Err(ProgramError::InvalidArgument);
    }

    let (name_account_key, key) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_name.clone(),
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

    let state = NameAuction::unpack_unchecked(&accounts.state.data.borrow())?;

    check_account_key(accounts.quote_mint, &Pubkey::new(&state.quote_mint))?;

    let (derived_state_key, derived_signer_nonce) =
        Pubkey::find_program_address(&[&name_account_key.to_bytes()], program_id);

    let signer_seeds: &[&[u8]] = &[&name_account_key.to_bytes(), &[derived_signer_nonce]];

    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    Cpi::claim_auction(
        accounts.spl_token_program,
        accounts.auction_program,
        accounts.clock_sysvar,
        accounts.auction,
        accounts.bonfida_vault,
        accounts.bidder_wallet,
        accounts.bidder_pot,
        accounts.bidder_pot_token,
        accounts.quote_mint,
        accounts.state,
        *accounts.name.key,
        signer_seeds,
    )?;

    Cpi::create_name_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.name,
        accounts.fee_payer,
        accounts.bidder_wallet,
        accounts.root_domain,
        accounts.central_state,
        hashed_name,
        lamports,
        space,
        signer_seeds,
    )?;

    Ok(())
}

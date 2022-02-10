use crate::{
    processor::BONFIDA_FIDA_VAULT,
    utils::{check_account_key, check_account_owner, check_signer, get_usd_price, Cpi},
};
use bonfida_utils::{fp_math::fp32_div, pyth::get_oracle_price_fp32};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::hashv,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::{self, Sysvar},
};
use spl_name_service::state::{get_seeds_and_key, NameRecordHeader, HASH_PREFIX};
use spl_token::instruction::transfer;

use super::{PYTH_FIDA_PRICE_ACC, ROOT_DOMAIN_ACCOUNT};

struct Accounts<'a, 'b: 'a> {
    rent_sysvar: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    reverse_lookup: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    buyer: &'a AccountInfo<'b>,
    buyer_token_source: &'a AccountInfo<'b>,
    pyth_fida_price_acc: &'a AccountInfo<'b>,
    fida_vault: &'a AccountInfo<'b>,
    spl_token_program: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        rent_sysvar: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        reverse_lookup: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        buyer: next_account_info(accounts_iter)?,
        buyer_token_source: next_account_info(accounts_iter)?,
        pyth_fida_price_acc: next_account_info(accounts_iter)?,
        fida_vault: next_account_info(accounts_iter)?,
        spl_token_program: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
    };

    // Check keys
    check_account_key(a.rent_sysvar, &sysvar::rent::id()).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_key(a.root_domain, &ROOT_DOMAIN_ACCOUNT).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_key(a.pyth_fida_price_acc, &PYTH_FIDA_PRICE_ACC).unwrap();
    check_account_key(a.fida_vault, &BONFIDA_FIDA_VAULT).unwrap();
    check_account_key(a.spl_token_program, &spl_token::ID).unwrap();

    // Check ownership
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_owner(a.central_state, program_id).unwrap();

    check_account_owner(a.state, &system_program::id()).unwrap();

    // Check signer
    check_signer(a.buyer).unwrap();

    Ok(a)
}

pub fn process_create_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    space: u32,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    if name != name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &name).as_bytes()])
        .as_ref()
        .to_vec();

    if hashed_name.len() != 32 {
        msg!("Invalid seed length");
        return Err(ProgramError::InvalidArgument);
    }

    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_name.clone(),
        None,
        Some(accounts.root_domain.key),
    );

    let signer_seeds = name_account_key.to_bytes();
    let (derived_state_key, _) = Pubkey::find_program_address(&[&signer_seeds], program_id);

    if &derived_state_key != accounts.state.key {
        msg!("An invalid name auctioning state account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    if !accounts.state.data_is_empty() {
        msg!("The name auctioning state account is not empty.");
        return Err(ProgramError::InvalidArgument);
    }

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }

    if accounts.name.data_len() != 0 {
        msg!("Name account is already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let hashed_reverse_lookup =
        hashv(&[(HASH_PREFIX.to_owned() + &name_account_key.to_string()).as_bytes()])
            .as_ref()
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

    let central_state_nonce = accounts.central_state.data.borrow()[0];

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    let min_price_fida = fp32_div(get_usd_price(name.len()), {
        #[cfg(feature = "mock-oracle")]
        {
            5 << 32
        }
        #[cfg(not(feature = "mock-oracle"))]
        get_oracle_price_fp32(&accounts.pyth_fida_price_acc.data.borrow(), 6, 6).unwrap()
    })
    .unwrap();

    // Transfer tokens
    let transfer_ix = transfer(
        &spl_token::ID,
        accounts.buyer_token_source.key,
        accounts.fida_vault.key,
        accounts.buyer.key,
        &[],
        min_price_fida,
    )?;

    invoke(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.buyer_token_source.clone(),
            accounts.fida_vault.clone(),
            accounts.buyer.clone(),
        ],
    )?;

    // Create domain name
    let rent = Rent::get()?;
    Cpi::create_name_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.name,
        accounts.buyer,
        accounts.buyer,
        accounts.root_domain,
        accounts.central_state,
        hashed_name,
        rent.minimum_balance(NameRecordHeader::LEN + space as usize),
        space,
        central_state_signer_seeds,
    )?;

    // Reverse look up
    if accounts.reverse_lookup.data_len() == 0 {
        Cpi::create_reverse_lookup_account(
            accounts.naming_service_program,
            accounts.system_program,
            accounts.reverse_lookup,
            accounts.buyer,
            name,
            hashed_reverse_lookup,
            accounts.central_state,
            accounts.rent_sysvar,
            central_state_signer_seeds,
        )?;
    }
    Ok(())
}

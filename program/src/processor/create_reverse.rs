use std::str::FromStr;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::hashv,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
    sysvar::{self},
};
use spl_name_service::state::{get_seeds_and_key, HASH_PREFIX};

use crate::utils::{check_account_key, check_account_owner, check_signer, Cpi};

use super::ROOT_DOMAIN_ACCOUNT;

struct Accounts<'a, 'b: 'a> {
    rent_sysvar: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    reverse_lookup: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
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
        reverse_lookup: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
    };

    check_account_key(a.rent_sysvar, &sysvar::rent::id()).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_owner(a.central_state, program_id).unwrap();
    check_account_key(
        a.root_domain,
        &Pubkey::from_str(ROOT_DOMAIN_ACCOUNT).unwrap(),
    )
    .unwrap();
    check_signer(a.fee_payer).unwrap();

    Ok(a)
}

pub fn process_create_reverse(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &name).as_bytes()])
        .as_ref()
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

    if accounts.reverse_lookup.data_len() == 0 {
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
    } else {
        msg!("Reverse lookup already exists. No-op");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

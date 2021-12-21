// Used to by pass the auction mechanism in order to reserve domain names for Solana ecosystem projects and prevent squatting

use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
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

use crate::{
    processor::{ADMIN_CREATE_KEY, ROOT_DOMAIN_ACCOUNT},
    utils::{check_account_key, check_account_owner, check_signer, Cpi},
};

struct Accounts<'a, 'b: 'a> {
    fee_payer: &'a AccountInfo<'b>,
    admin: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    reverse_lookup: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    rent_sysvar: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
    params: &Params,
) -> Result<(Accounts<'a, 'b>, Vec<u8>, Vec<u8>), ProgramError> {
    let accounts_iter = &mut accounts.iter();

    let accounts = Accounts {
        fee_payer: next_account_info(accounts_iter)?,
        admin: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        reverse_lookup: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        rent_sysvar: next_account_info(accounts_iter)?,
    };
    check_signer(accounts.fee_payer)?;
    check_signer(accounts.admin)?;
    check_account_key(accounts.admin, &Pubkey::from_str(ADMIN_CREATE_KEY).unwrap()).unwrap();
    check_account_key(accounts.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_owner(accounts.root_domain, &spl_name_service::id()).unwrap();
    check_account_key(
        accounts.root_domain,
        &Pubkey::from_str(ROOT_DOMAIN_ACCOUNT).unwrap(),
    )
    .unwrap();
    check_account_owner(accounts.central_state, program_id).unwrap();
    check_account_key(accounts.system_program, &system_program::id()).unwrap();
    check_account_key(accounts.rent_sysvar, &sysvar::rent::id()).unwrap();

    if params.name != params.name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &params.name).as_bytes()])
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

    check_account_key(accounts.name, &name_account_key).unwrap();

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

    check_account_key(accounts.reverse_lookup, &reverse_lookup_account_key).unwrap();

    Ok((accounts, hashed_name, hashed_reverse_lookup))
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct Params {
    lamports: u64,
    space: u32,
    name: String,
}

pub fn process_create_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let (accounts, hashed_name, hashed_reverse_lookup) =
        parse_accounts(program_id, accounts, &params)?;

    let Params {
        lamports,
        space,
        name,
    } = params;

    let central_state_nonce = accounts.central_state.data.borrow()[0];

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    Cpi::create_name_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.name,
        accounts.fee_payer,
        accounts.admin,
        accounts.root_domain,
        accounts.central_state,
        hashed_name,
        lamports,
        space,
        central_state_signer_seeds,
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

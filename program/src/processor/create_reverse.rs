use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::hashv,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
    sysvar::{self},
};
use spl_name_service::state::{get_seeds_and_key, NameRecordHeader, HASH_PREFIX};

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
    parent_name_opt: Option<&'a AccountInfo<'b>>,
    parent_name_owner_opt: Option<&'a AccountInfo<'b>>,
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
        parent_name_opt: next_account_info(accounts_iter).ok(),
        parent_name_owner_opt: next_account_info(accounts_iter).ok(),
    };

    // Check keys
    check_account_key(a.rent_sysvar, &sysvar::rent::id()).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_key(a.root_domain, &ROOT_DOMAIN_ACCOUNT).unwrap();

    // Check owners
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_owner(a.central_state, program_id).unwrap();

    // Check signer
    check_signer(a.fee_payer).unwrap();
    a.parent_name_owner_opt
        .map(check_signer)
        .transpose()
        .unwrap();

    Ok(a)
}

pub fn process_create_reverse(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    if let Some(parent_name) = accounts.parent_name_opt {
        assert!(accounts.parent_name_owner_opt.is_some());
        check_account_owner(parent_name, &spl_name_service::ID).unwrap();
        check_signer(accounts.parent_name_owner_opt.unwrap()).unwrap();
        let parent = NameRecordHeader::unpack_from_slice(&parent_name.data.borrow()).unwrap();
        assert_eq!(parent.parent_name, ROOT_DOMAIN_ACCOUNT);
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
        hashed_name,
        None,
        accounts
            .parent_name_opt
            .map_or(Some(&ROOT_DOMAIN_ACCOUNT), |acc| Some(acc.key)),
    );

    let hashed_reverse_lookup =
        hashv(&[(HASH_PREFIX.to_owned() + &name_account_key.to_string()).as_bytes()])
            .as_ref()
            .to_vec();

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(accounts.central_state.key),
        accounts.parent_name_opt.map(|a| a.key),
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
            accounts.parent_name_opt,
            accounts.parent_name_owner_opt,
        )?;
    } else {
        msg!("Reverse lookup already exists. No-op");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

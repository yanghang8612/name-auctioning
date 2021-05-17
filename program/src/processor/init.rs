use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    state::CentralState,
    utils::{check_account_key, check_account_owner, Cpi},
};

struct Accounts<'a, 'b: 'a> {
    state_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
    rent_sysvar_account: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        state_account: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
        rent_sysvar_account: next_account_info(accounts_iter)?,
    };

    check_account_owner(a.state_account, &system_program::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_key(a.rent_sysvar_account, &sysvar::rent::id()).unwrap();

    Ok(a)
}

pub fn process_init(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    state_nonce: u8,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    let signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[state_nonce]];
    let derived_state_key = Pubkey::create_program_address(signer_seeds, program_id)?;

    if &derived_state_key != accounts.state_account.key {
        msg!("Incorrect state account or signer nonce provided");
        return Err(ProgramError::InvalidArgument);
    }

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.state_account,
        accounts.rent_sysvar_account,
        signer_seeds,
        CentralState::LEN,
    )?;

    let state = CentralState {
        signer_nonce: state_nonce,
    };
    state.pack_into_slice(&mut accounts.state_account.data.borrow_mut());

    Ok(())
}

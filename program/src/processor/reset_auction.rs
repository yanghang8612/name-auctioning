use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

use crate::utils::{check_account_key, check_account_owner, check_signer, Cpi};

use super::{ADMIN, AUCTION_PROGRAM_ID};

struct Accounts<'a, 'b: 'a> {
    auction_program: &'a AccountInfo<'b>,
    clock_sysvar: &'a AccountInfo<'b>,
    admin: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        auction_program: next_account_info(accounts_iter)?,
        clock_sysvar: next_account_info(accounts_iter)?,
        admin: next_account_info(accounts_iter)?,
        auction: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
    };

    check_account_key(a.auction_program, &AUCTION_PROGRAM_ID).unwrap();
    check_account_key(a.clock_sysvar, &sysvar::clock::id()).unwrap();
    check_account_owner(a.auction, &AUCTION_PROGRAM_ID).unwrap();
    check_account_owner(a.state, program_id).unwrap();

    #[cfg(not(feature = "no-admin"))]
    check_account_key(a.admin, &ADMIN).unwrap();
    check_signer(a.admin).unwrap();

    Ok(a)
}

pub fn process_reset_auction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    let signer_seeds = accounts.name.key.to_bytes();

    let (derived_state_key, derived_signer_nonce) =
        Pubkey::find_program_address(&[&signer_seeds], program_id);

    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let signer_seeds: &[&[u8]] = &[&signer_seeds, &[derived_signer_nonce]];

    Cpi::start_auction(
        accounts.auction_program,
        accounts.clock_sysvar,
        accounts.auction,
        accounts.state,
        *accounts.name.key,
        signer_seeds,
    )?;
    Ok(())
}

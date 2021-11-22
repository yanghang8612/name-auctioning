use std::str::FromStr;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_auction::{instruction::close_auction_pot, processor::AuctionData};

use super::ADMIN_CLAIM_KEY;
use crate::{
    error::NameAuctionError,
    state::NameAuction,
    utils::{check_account_key, check_signer, Cpi},
};

struct Accounts<'a, 'b: 'a> {
    clock_sysvar: &'a AccountInfo<'b>,
    spl_token_program: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    quote_mint: &'a AccountInfo<'b>,
    bidder_wallet: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    bidder_pot_token: &'a AccountInfo<'b>,
    bonfida_vault: &'a AccountInfo<'b>,
    admin_signer: &'a AccountInfo<'b>,
    new_name_owner: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        clock_sysvar: next_account_info(accounts_iter)?,
        spl_token_program: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        auction_program: next_account_info(accounts_iter)?,
        auction: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
        quote_mint: next_account_info(accounts_iter)?,
        bidder_wallet: next_account_info(accounts_iter)?,
        bidder_pot: next_account_info(accounts_iter)?,
        bidder_pot_token: next_account_info(accounts_iter)?,
        bonfida_vault: next_account_info(accounts_iter)?,
        admin_signer: next_account_info(accounts_iter)?,
        new_name_owner: next_account_info(accounts_iter)?,
    };
    check_signer(a.admin_signer).unwrap();
    check_account_key(a.admin_signer, &Pubkey::from_str(ADMIN_CLAIM_KEY).unwrap()).unwrap();

    Ok(a)
}

pub fn process_a_claim(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = parse_accounts(accounts)?;

    let state = NameAuction::unpack_unchecked(&accounts.state.data.borrow())?;

    check_account_key(accounts.quote_mint, &Pubkey::new(&state.quote_mint))?;

    let (derived_state_key, _) =
        Pubkey::find_program_address(&[&accounts.name.key.to_bytes()], program_id);
    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }
    if accounts.name.data_is_empty() {
        msg!("Name account does not exist");
        return Err(ProgramError::InvalidArgument);
    }

    let central_state_nonce = accounts.central_state.data.borrow()[0];

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    let auction: AuctionData = try_from_slice_unchecked(&accounts.auction.data.borrow()).unwrap();
    let clock = Clock::from_account_info(accounts.clock_sysvar).unwrap();

    if !auction.ended(clock.unix_timestamp).unwrap() {
        msg!("The auction must have ended to reclaim");
        return Err(NameAuctionError::AuctionInProgress.into());
    }

    if let spl_auction::processor::BidState::EnglishAuction { bids, max: _ } = auction.bid_state {
        if bids.is_empty() {
            msg!("The auction has no bidder and cannot be a reclaimed!");
            return Err(ProgramError::InvalidArgument);
        }
    } else {
        unreachable!()
    }

    Cpi::transfer_name_account(
        accounts.naming_service_program,
        accounts.central_state,
        accounts.name,
        &accounts.new_name_owner.key,
        Some(central_state_signer_seeds),
    )?;

    let clean_up_instr = close_auction_pot(
        *accounts.auction_program.key,
        *accounts.auction.key,
        *accounts.bidder_pot.key,
        *accounts.bidder_wallet.key,
        *accounts.bonfida_vault.key,
        *accounts.system_program.key,
        *accounts.central_state.key,
        *accounts.bidder_pot_token.key,
        *accounts.bonfida_vault.key,
        *accounts.name.key,
    );
    invoke_signed(
        &clean_up_instr,
        &[
            accounts.auction_program.clone(),
            accounts.auction.clone(),
            accounts.bidder_pot.clone(),
            accounts.bidder_pot_token.clone(),
            accounts.bidder_wallet.clone(),
            accounts.bonfida_vault.clone(),
            accounts.system_program.clone(),
            accounts.central_state.clone(),
            accounts.name.clone(),
            accounts.spl_token_program.clone(),
        ],
        &[central_state_signer_seeds],
    )?;

    Ok(())
}

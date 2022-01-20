use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_auction::{instruction::close_auction_pot, processor::AuctionData};

use super::ADMIN_CLAIM_KEY;
use crate::{
    error::NameAuctionError,
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
    bidder_wallet: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    bidder_pot_token: &'a AccountInfo<'b>,
    bonfida_vault: &'a AccountInfo<'b>,
    admin_signer: &'a AccountInfo<'b>,
    new_name_owner: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
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
        bidder_wallet: next_account_info(accounts_iter)?,
        bidder_pot: next_account_info(accounts_iter)?,
        bidder_pot_token: next_account_info(accounts_iter)?,
        bonfida_vault: next_account_info(accounts_iter)?,
        admin_signer: next_account_info(accounts_iter)?,
        new_name_owner: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
    };
    check_signer(a.admin_signer).unwrap();
    check_account_key(a.admin_signer, &ADMIN_CLAIM_KEY).unwrap();

    Ok(a)
}

pub fn process_a_claim(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    hashed_name: Vec<u8>,
    lamports: u64,
    space: u32,
) -> ProgramResult {
    let accounts = parse_accounts(accounts)?;

    let (derived_state_key, _) =
        Pubkey::find_program_address(&[&accounts.name.key.to_bytes()], program_id);
    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let central_state_nonce = accounts.central_state.data.borrow()[0];
    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    if accounts.name.data_is_empty() {
        msg!("Name account does not exist. Creating.");
        Cpi::create_name_account(
            accounts.naming_service_program,
            accounts.system_program,
            accounts.name,
            accounts.fee_payer,
            accounts.new_name_owner,
            accounts.root_domain,
            accounts.central_state,
            hashed_name,
            lamports,
            space,
            central_state_signer_seeds,
        )?;
    } else {
        Cpi::transfer_name_account(
            accounts.naming_service_program,
            accounts.central_state,
            accounts.name,
            accounts.new_name_owner.key,
            Some(central_state_signer_seeds),
        )?;
    }

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

    let clean_up_instr = close_auction_pot(
        *accounts.auction_program.key,
        *accounts.auction.key,
        *accounts.bidder_pot.key,
        *accounts.bidder_wallet.key,
        *accounts.bonfida_vault.key,
        *accounts.system_program.key,
        *accounts.central_state.key,
        *accounts.bonfida_vault.key,
        *accounts.bidder_pot_token.key,
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

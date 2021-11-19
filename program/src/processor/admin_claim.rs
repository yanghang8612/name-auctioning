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
use spl_name_service::state::get_seeds_and_key;
use spl_token::state::Account;

use super::{ADMIN_CLAIM_KEY, BONFIDA_FIDA_VAULT, BONFIDA_USDC_VAULT, FEES, FEE_TIERS, FIDA_MINT};
use crate::{
    error::NameAuctionError,
    state::{NameAuction, ResellingAuction},
    utils::{check_account_key, check_account_owner, check_signer, Cpi},
};

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
    reselling_state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
    quote_mint: &'a AccountInfo<'b>,
    destination_token: &'a AccountInfo<'b>,
    bidder_wallet: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    bidder_pot_token: &'a AccountInfo<'b>,
    bonfida_vault: &'a AccountInfo<'b>,
    fida_discount: &'a AccountInfo<'b>,
    buy_now: &'a AccountInfo<'b>,
    bonfida_sol_vault: &'a AccountInfo<'b>,
    admin_signer: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
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
        reselling_state: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
        quote_mint: next_account_info(accounts_iter)?,
        destination_token: next_account_info(accounts_iter)?,
        bidder_wallet: next_account_info(accounts_iter)?,
        bidder_pot: next_account_info(accounts_iter)?,
        bidder_pot_token: next_account_info(accounts_iter)?,
        bonfida_vault: next_account_info(accounts_iter)?,
        fida_discount: next_account_info(accounts_iter)?,
        buy_now: next_account_info(accounts_iter)?,
        bonfida_sol_vault: next_account_info(accounts_iter)?,
        admin_signer: next_account_info(accounts_iter)?,
    };
    check_signer(a.admin_signer).unwrap();
    check_account_key(a.admin_signer, &Pubkey::from_str(ADMIN_CLAIM_KEY).unwrap()).unwrap();

    Ok(a)
}

pub fn process_claim(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    hashed_name: Vec<u8>,
    lamports: u64,
    space: u32,
) -> ProgramResult {
    let accounts = parse_accounts(accounts)?;

    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_name.clone(),
        None,
        Some(accounts.root_domain.key),
    );

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }

    let state = NameAuction::unpack_unchecked(&accounts.state.data.borrow())?;

    check_account_key(accounts.quote_mint, &Pubkey::new(&state.quote_mint))?;

    let (derived_state_key, derived_signer_nonce) =
        Pubkey::find_program_address(&[&name_account_key.to_bytes()], program_id);
    if &derived_state_key != accounts.state.key {
        msg!("An invalid signer account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let signer_seeds: &[&[u8]] = &[&name_account_key.to_bytes(), &[derived_signer_nonce]];

    let central_state_nonce = accounts.central_state.data.borrow()[0];

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    let mut fee_percentage = 0;
    if accounts.name.data_is_empty() {
        check_account_key(
            accounts.destination_token,
            &Pubkey::from_str(BONFIDA_FIDA_VAULT).unwrap(),
        )
        .or_else(|_| {
            check_account_key(
                accounts.destination_token,
                &Pubkey::from_str(BONFIDA_USDC_VAULT).unwrap(),
            )
        })
        .unwrap();
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
            central_state_signer_seeds,
        )?;
    } else {
        // Claiming a reselling auction
        let reselling_state =
            ResellingAuction::unpack_unchecked(&accounts.reselling_state.data.borrow())?;

        let (derived_reselling_state_key, _) =
            Pubkey::find_program_address(&[&name_account_key.to_bytes(), &[1u8, 1u8]], program_id);
        if &derived_reselling_state_key != accounts.reselling_state.key {
            msg!("An incorrect reselling state account was provided");
            return Err(ProgramError::InvalidArgument);
        }
        check_account_owner(accounts.reselling_state, &program_id).unwrap();
        check_account_key(
            accounts.destination_token,
            &Pubkey::new(&reselling_state.token_destination_account),
        )
        .unwrap();

        let auction: AuctionData =
            try_from_slice_unchecked(&accounts.auction.data.borrow()).unwrap();
        let clock = Clock::from_account_info(accounts.clock_sysvar).unwrap();

        if !auction.ended(clock.unix_timestamp).unwrap() {
            msg!("The auction must have ended to reclaim");
            return Err(NameAuctionError::AuctionInProgress.into());
        }

        if let spl_auction::processor::BidState::EnglishAuction { bids, max: _ } = auction.bid_state
        {
            if bids.is_empty() {
                msg!("The auction has no bidder and can be reclaimed!");
                let token_destination_account_owner =
                    spl_token::state::Account::unpack(&accounts.destination_token.data.borrow())?;
                check_account_key(
                    accounts.bidder_wallet,
                    &token_destination_account_owner.owner,
                )?;

                Cpi::transfer_name_account(
                    accounts.naming_service_program,
                    accounts.central_state,
                    accounts.name,
                    &accounts.bidder_wallet.key,
                    Some(central_state_signer_seeds),
                )?;
                return Ok(());
            }
        } else {
            unreachable!()
        }

        Cpi::transfer_name_account(
            accounts.naming_service_program,
            accounts.central_state,
            accounts.name,
            &accounts.bidder_wallet.key,
            Some(central_state_signer_seeds),
        )?;

        // Calculate fees
        let mut fee_tier = 0;

        if let Ok(discount_data) = Account::unpack(&accounts.fida_discount.data.borrow()) {
            let destination_data = Account::unpack(&accounts.destination_token.data.borrow())?;
            if discount_data.owner != destination_data.owner {
                msg!("Fida discount owner does not match destination owner.");
                return Err(ProgramError::InvalidArgument);
            }
            if discount_data.mint.to_string() != FIDA_MINT {
                msg!("The discount account should be a FIDA token account");
                return Err(ProgramError::InvalidArgument);
            }
            fee_tier = match FEE_TIERS
                .iter()
                .position(|&t| discount_data.amount < (t as u64))
            {
                Some(i) => i,
                None => FEE_TIERS.len(),
            };
        }

        fee_percentage = FEES[fee_tier];
    }

    let clean_up_instr = close_auction_pot(
        *accounts.auction_program.key,
        *accounts.auction.key,
        *accounts.bidder_pot.key,
        *accounts.bidder_wallet.key,
        *accounts.bonfida_vault.key,
        *accounts.system_program.key,
        *accounts.central_state.key,
        *accounts.name.key,
    );
    invoke_signed(
        &clean_up_instr,
        &[
            accounts.auction_program.clone(),
            accounts.auction.clone(),
            accounts.bidder_pot.clone(),
            accounts.bidder_wallet.clone(),
            accounts.bonfida_vault.clone(),
            accounts.system_program.clone(),
            accounts.central_state.clone(),
            accounts.name.clone(),
        ],
        &[central_state_signer_seeds],
    );

    Cpi::claim_auction(
        accounts.spl_token_program,
        accounts.auction_program,
        accounts.clock_sysvar,
        accounts.auction,
        accounts.destination_token,
        accounts.bidder_wallet,
        accounts.bidder_pot,
        accounts.bidder_pot_token,
        accounts.quote_mint,
        accounts.state,
        accounts.bonfida_vault,
        accounts.buy_now,
        accounts.bonfida_sol_vault,
        *accounts.name.key,
        signer_seeds,
        fee_percentage,
    )?;

    Ok(())
}

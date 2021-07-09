use std::str::FromStr;

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    hash::hashv,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
    sysvar::{self, Sysvar},
};
use spl_auction::processor::{AuctionData, BidState};
use spl_name_service::state::{get_seeds_and_key, NameRecordHeader, HASH_PREFIX};

use crate::{
    error::NameAuctionError,
    processor::TOKEN_MINT,
    state::{NameAuction, NameAuctionStatus, ResellingAuction},
    utils::{check_account_key, check_account_owner, check_signer, Cpi},
};
use spl_token::state::Account;

use super::AUCTION_PROGRAM_ID;

struct Accounts<'a, 'b: 'a> {
    rent_sysvar: &'a AccountInfo<'b>,
    clock_sysvar: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    name_owner: &'a AccountInfo<'b>,
    reverse_lookup: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
    reselling_state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    token_destination_account: &'a AccountInfo<'b>,
    fee_payer: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        rent_sysvar: next_account_info(accounts_iter)?,
        clock_sysvar: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        root_domain: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        name_owner: next_account_info(accounts_iter)?,
        reverse_lookup: next_account_info(accounts_iter)?,
        system_program: next_account_info(accounts_iter)?,
        auction_program: next_account_info(accounts_iter)?,
        auction: next_account_info(accounts_iter)?,
        central_state: next_account_info(accounts_iter)?,
        state: next_account_info(accounts_iter)?,
        reselling_state: next_account_info(accounts_iter)?,
        token_destination_account: next_account_info(accounts_iter)?,
        fee_payer: next_account_info(accounts_iter)?,
    };

    check_account_key(a.rent_sysvar, &sysvar::rent::id()).unwrap();
    check_account_key(a.clock_sysvar, &sysvar::clock::id()).unwrap();
    check_signer(a.name_owner).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_owner(a.root_domain, &spl_name_service::id()).unwrap();
    check_account_key(a.system_program, &system_program::id()).unwrap();
    check_account_owner(a.central_state, &program_id).unwrap();
    check_account_key(
        a.auction_program,
        &Pubkey::from_str(AUCTION_PROGRAM_ID).unwrap(),
    )
    .unwrap();
    // check_account_owner(a.auction, &spl_auction::id()).unwrap();
    check_account_owner(a.state, &system_program::id())
        .or_else(|_| check_account_owner(a.state, program_id))
        .unwrap();
    check_account_owner(a.reselling_state, &system_program::id())
        .or_else(|_| check_account_owner(a.reselling_state, program_id))
        .unwrap();
    check_signer(a.fee_payer).unwrap();

    Ok(a)
}

pub fn process_resell(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    minimum_price: u64,
    en_auction_at: u64,
) -> ProgramResult {
    let accounts = parse_accounts(program_id, accounts)?;

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &name).as_bytes()])
        .0
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

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }
    let name_record = NameRecordHeader::unpack_from_slice(&accounts.name.data.borrow()).unwrap();
    if accounts.name.data_len() == 0 {
        msg!("Name account is not initialized. Please create an auction before reselling.");
        return Err(ProgramError::UninitializedAccount);
    }
    let token_destination_account =
        Account::unpack(&accounts.token_destination_account.data.borrow())?;
    if Pubkey::from_str(TOKEN_MINT).unwrap() != token_destination_account.mint {
        msg!("Destination token account is not of the right mint.");
        return Err(ProgramError::InvalidArgument);
    }

    let signer_seeds = name_account_key.to_bytes();

    let (derived_state_key, derived_state_signer_nonce) =
        Pubkey::find_program_address(&[&signer_seeds], program_id);
    if &derived_state_key != accounts.state.key {
        msg!("An invalid state account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let (derived_reselling_state_key, derived_reselling_signer_nonce) =
        Pubkey::find_program_address(&[&signer_seeds, &[1u8, 1u8]], program_id);

    if &derived_reselling_state_key != accounts.reselling_state.key {
        msg!("An invalid reselling state account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    let state_signer_seeds: &[&[u8]] = &[&signer_seeds, &[derived_state_signer_nonce]];
    let reselling_state_signer_seeds: &[&[u8]] = &[
        &signer_seeds,
        &[1u8, 1u8],
        &[derived_reselling_signer_nonce],
    ];

    if accounts.state.data_len() != 0 {
        msg!("An auction for this name has already been created.");
        let state = NameAuction::unpack_unchecked(&accounts.state.data.borrow()).unwrap();
        if accounts.auction.key != &Pubkey::new(&state.auction_account) {
            msg!("Provided invalid auction account");
            return Err(ProgramError::InvalidArgument);
        }

        if name_record.owner == *accounts.central_state.key {
            let current_timestamp = Clock::from_account_info(accounts.clock_sysvar)?.unix_timestamp;
            let auction: AuctionData =
                try_from_slice_unchecked(&accounts.auction.data.borrow()).unwrap();

            if !auction.ended(current_timestamp)? {
                msg!("The auction has to end before it can be restarted!");
                return Err(NameAuctionError::AuctionInProgress.into());
            }

            match state.status {
                NameAuctionStatus::FirstAuction => {
                    msg!("This is not a reselling auction. Please restart it with the create instruction, or claim it!");
                    return Err(ProgramError::InvalidArgument);
                }
                NameAuctionStatus::SecondaryAuction => {
                    if let BidState::EnglishAuction { bids, max: _ } = auction.bid_state {
                        if !bids.is_empty() {
                            msg!("The auction has a bidder, which means it has a winner and cannot be reset!");
                            return Err(NameAuctionError::AuctionRealized.into());
                        }
                        msg!("Restarting auction.");
                        Cpi::start_auction(
                            accounts.auction_program,
                            accounts.clock_sysvar,
                            accounts.auction,
                            accounts.state,
                            *accounts.name.key,
                            &state_signer_seeds,
                        )?;
                        return Ok(());
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    let hashed_reverse_lookup =
        hashv(&[(HASH_PREFIX.to_owned() + &name_account_key.to_string()).as_bytes()])
            .0
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

    if accounts.state.data_len() == 0 {
        Cpi::create_account(
            program_id,
            accounts.system_program,
            accounts.fee_payer,
            accounts.state,
            accounts.rent_sysvar,
            state_signer_seeds,
            NameAuction::LEN,
        )?;
    }
    if accounts.reselling_state.data_len() == 0 {
        Cpi::create_account(
            program_id,
            accounts.system_program,
            accounts.fee_payer,
            accounts.reselling_state,
            accounts.rent_sysvar,
            reselling_state_signer_seeds,
            ResellingAuction::LEN,
        )?;
    }

    let state = NameAuction {
        status: NameAuctionStatus::SecondaryAuction,
        quote_mint: Pubkey::from_str(TOKEN_MINT).unwrap().to_bytes(),
        signer_nonce: derived_reselling_signer_nonce,
        auction_account: accounts.auction.key.to_bytes(),
    };

    {
        let mut pt: &mut [u8] = &mut accounts.state.data.borrow_mut();
        state.serialize(&mut pt)?;
    }

    let reselling_state = ResellingAuction {
        token_destination_account: accounts.token_destination_account.key.to_bytes(),
    };

    {
        let mut pt: &mut [u8] = &mut accounts.reselling_state.data.borrow_mut();
        reselling_state.serialize(&mut pt)?;
    }

    msg!("Setting up auction");
    solana_program::log::sol_log_compute_units();

    Cpi::create_auction(
        accounts.auction_program,
        accounts.rent_sysvar,
        accounts.system_program,
        accounts.auction,
        accounts.fee_payer,
        Some(en_auction_at),
        accounts.state,
        *accounts.name.key,
        minimum_price,
    )?;

    msg!("Transferring the domain ownership to the auction program");

    Cpi::transfer_name_account(
        accounts.naming_service_program,
        accounts.name_owner,
        accounts.name,
        &accounts.central_state.key,
        None,
    )?;

    solana_program::log::sol_log_compute_units();

    msg!("Starting auction");

    Cpi::start_auction(
        accounts.auction_program,
        accounts.clock_sysvar,
        accounts.auction,
        accounts.state,
        *accounts.name.key,
        &state_signer_seeds,
    )?;

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
    }

    Ok(())
}

use std::str::FromStr;

use crate::processor::BONFIDA_SOL_VAULT;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    hash::hashv,
    msg,
    native_token::LAMPORTS_PER_SOL,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{self},
};
use spl_auction::processor::{AuctionData, BidState};
use spl_name_service::state::{get_seeds_and_key, HASH_PREFIX};
use spl_token::state::Account;

use crate::{
    state::{NameAuction, NameAuctionStatus, ResellingAuction},
    utils::{check_account_key, check_account_owner, check_signer, Cpi},
};

use super::AUCTION_PROGRAM_ID;

struct Accounts<'a, 'b: 'a> {
    clock_sysvar: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    root_domain: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    central_state: &'a AccountInfo<'b>,
    state: &'a AccountInfo<'b>,
    auction_program: &'a AccountInfo<'b>,
    auction_creator: &'a AccountInfo<'b>,
    reselling_state: &'a AccountInfo<'b>,
    destination_token: &'a AccountInfo<'b>,
    bonfida_sol_vault: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
}

impl<'a, 'b: 'a> Accounts<'a, 'b> {
    pub fn parse(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
        name: String,
    ) -> Result<(Accounts<'a, 'b>, [u8; 32], u8), ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            clock_sysvar: next_account_info(accounts_iter)?,
            naming_service_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            name: next_account_info(accounts_iter)?,
            auction_program: next_account_info(accounts_iter)?,
            auction: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            state: next_account_info(accounts_iter)?,
            auction_creator: next_account_info(accounts_iter)?,
            reselling_state: next_account_info(accounts_iter)?,
            destination_token: next_account_info(accounts_iter)?,
            bonfida_sol_vault: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        let spl_auction_id = &Pubkey::from_str(AUCTION_PROGRAM_ID).unwrap();

        // Params check and derivations
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

        let signer_seeds = name_account_key.to_bytes();

        let (derived_state_key, derived_signer_nonce) =
            Pubkey::find_program_address(&[&signer_seeds], program_id);

        let reselling_state =
            ResellingAuction::unpack_unchecked(&accounts.reselling_state.data.borrow())?;

        let (derived_reselling_state_key, _) =
            Pubkey::find_program_address(&[&name_account_key.to_bytes(), &[1u8, 1u8]], program_id);

        let destination_account = Account::unpack(&accounts.destination_token.data.borrow())?;
        let state = NameAuction::unpack_unchecked(&accounts.state.data.borrow()).unwrap();

        if state.status == NameAuctionStatus::FirstAuction {
            msg!("Cannot reset primary auction");
            return Err(ProgramError::InvalidArgument);
        }

        let auction: AuctionData =
            try_from_slice_unchecked(&accounts.auction.data.borrow()).unwrap();
        let bids_empty = match auction.bid_state {
            BidState::EnglishAuction { bids, max: _ } => bids.is_empty(),
            _ => unreachable!(),
        };

        if !bids_empty {
            msg!("Cannot cancel auctions with bids");
            return Err(ProgramError::InvalidArgument);
        }

        // Key checks
        check_account_key(accounts.clock_sysvar, &sysvar::clock::id()).unwrap();
        check_account_key(accounts.naming_service_program, &spl_name_service::id()).unwrap();
        check_account_key(accounts.auction_program, spl_auction_id).unwrap();
        check_account_key(accounts.name, &name_account_key).unwrap();
        check_account_key(accounts.state, &derived_state_key).unwrap();
        check_account_key(accounts.reselling_state, &derived_reselling_state_key).unwrap();
        check_account_key(
            accounts.destination_token,
            &Pubkey::new(&reselling_state.token_destination_account),
        )
        .unwrap();
        check_account_key(accounts.auction_creator, &destination_account.owner).unwrap();
        check_account_key(
            accounts.bonfida_sol_vault,
            &Pubkey::from_str(BONFIDA_SOL_VAULT).unwrap(),
        )
        .unwrap();

        // Signer checks
        check_signer(accounts.auction_creator).unwrap();

        // Ownership checks
        check_account_owner(accounts.auction, spl_auction_id).unwrap();
        check_account_owner(accounts.central_state, program_id).unwrap();
        check_account_owner(accounts.state, program_id).unwrap();
        check_account_owner(accounts.root_domain, &spl_name_service::id()).unwrap();
        check_account_owner(accounts.reselling_state, program_id).unwrap();

        Ok((accounts, signer_seeds, derived_signer_nonce))
    }
}

pub fn process_end_auction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
) -> ProgramResult {
    let (accounts, signer_seeds, derived_signer_nonce) =
        Accounts::parse(program_id, accounts, name)?;

    let signer_seeds: &[&[u8]] = &[&signer_seeds, &[derived_signer_nonce]];

    msg!("Ending auction");
    Cpi::end_auction(
        accounts.auction_program,
        accounts.state,
        accounts.auction,
        accounts.clock_sysvar,
        *accounts.name.key,
        signer_seeds,
    )?;

    let central_state_nonce = accounts.central_state.data.borrow()[0];
    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    msg!("Transferring domain names to auction creator");
    Cpi::transfer_name_account(
        accounts.naming_service_program,
        accounts.central_state,
        accounts.name,
        accounts.auction_creator.key,
        Some(central_state_signer_seeds),
    )?;

    // Charge a 0.5 SOL fee for users cancelling auctions
    let fee_instruction = system_instruction::transfer(
        accounts.auction_creator.key,
        accounts.bonfida_sol_vault.key,
        (LAMPORTS_PER_SOL / 2) as u64,
    );
    invoke(
        &fee_instruction,
        &[
            accounts.system_program.clone(),
            accounts.auction_creator.clone(),
            accounts.bonfida_sol_vault.clone(),
        ],
    )?;

    Ok(())
}

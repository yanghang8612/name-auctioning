use std::str::FromStr;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction::create_account, sysvar::Sysvar,
};
use spl_auction::{
    instruction::{claim_bid_instruction, create_auction_instruction, start_auction_instruction},
    processor::{ClaimBidArgs, CreateAuctionArgs, PriceFloor, StartAuctionArgs, WinnerLimit},
};
use spl_name_service::instruction::NameRegistryInstruction;

use crate::{
    processor::{END_AUCTION_GAP, MINIMUM_PRICE, TOKEN_MINT},
    state::NameAuction,
};

pub struct Cpi {}

impl Cpi {
    pub fn create_account<'a>(
        program_id: &Pubkey,
        system_program: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        account_to_create: &AccountInfo<'a>,
        rent_sysvar_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        space: usize,
    ) -> ProgramResult {
        let rent = Rent::from_account_info(rent_sysvar_account)?;

        let create_state_instruction = create_account(
            fee_payer.key,
            account_to_create.key,
            rent.minimum_balance(NameAuction::LEN),
            space as u64,
            program_id,
        );

        invoke_signed(
            &create_state_instruction,
            &[
                system_program.clone(),
                fee_payer.clone(),
                account_to_create.clone(),
            ],
            &[signer_seeds],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_auction<'a>(
        auction_program: &AccountInfo<'a>,
        rent_sysvar_account: &AccountInfo<'a>,
        system_account: &AccountInfo<'a>,
        auction_account: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        end_auction_at: Option<u64>,
        authority: &AccountInfo<'a>,
        resource: Pubkey,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let create_auction_instruction = create_auction_instruction(
            *auction_program.key,
            *fee_payer.key,
            CreateAuctionArgs {
                winners: WinnerLimit::Capped(1),
                end_auction_at,
                end_auction_gap: Some(END_AUCTION_GAP),
                token_mint: Pubkey::from_str(TOKEN_MINT).unwrap(),
                authority: *authority.key,
                resource,
                price_floor: PriceFloor::MinimumPrice([MINIMUM_PRICE, 0, 0, 0]),
            },
        );

        invoke_signed(
            &create_auction_instruction,
            &[
                auction_program.clone(),
                authority.clone(),
                auction_account.clone(),
                rent_sysvar_account.clone(),
                system_account.clone(),
            ],
            &[signer_seeds],
        )
    }

    pub fn start_auction<'a>(
        auction_program: &AccountInfo<'a>,
        clock_sysvar_account: &AccountInfo<'a>,
        auction_account: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        resource: Pubkey,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let create_auction_instruction = start_auction_instruction(
            *auction_program.key,
            *authority.key,
            StartAuctionArgs { resource },
        );

        invoke_signed(
            &create_auction_instruction,
            &[
                auction_program.clone(),
                authority.clone(),
                auction_account.clone(),
                clock_sysvar_account.clone(),
            ],
            &[signer_seeds],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn claim_auction<'a>(
        spl_token_program: &AccountInfo<'a>,
        auction_program: &AccountInfo<'a>,
        clock_sysvar_account: &AccountInfo<'a>,
        auction_account: &AccountInfo<'a>,
        bonfida_vault: &AccountInfo<'a>,
        winner_account: &AccountInfo<'a>,
        winner_pot_account: &AccountInfo<'a>,
        winner_pot_token_account: &AccountInfo<'a>,
        quote_mint: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        resource: Pubkey,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let claim_auction_instruction = claim_bid_instruction(
            *auction_program.key,
            *authority.key,
            *bonfida_vault.key,
            *winner_account.key,
            *winner_pot_token_account.key,
            *quote_mint.key,
            ClaimBidArgs { resource },
        );

        invoke_signed(
            &claim_auction_instruction,
            &[
                auction_program.clone(),
                bonfida_vault.clone(),
                winner_pot_token_account.clone(),
                winner_pot_account.clone(),
                authority.clone(),
                auction_account.clone(),
                winner_account.clone(),
                quote_mint.clone(),
                clock_sysvar_account.clone(),
                spl_token_program.clone(),
            ],
            &[signer_seeds],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        new_owner_account: &AccountInfo<'a>,
        root_name_account: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        hashed_name: Vec<u8>,
        lamports: u64,
        space: u32,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let create_name_instruction = spl_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name,
                lamports,
                space,
            },
            *name_account.key,
            *fee_payer.key,
            *new_owner_account.key,
            None,
            Some(*root_name_account.key),
            Some(*authority.key),
        )?;

        invoke_signed(
            &create_name_instruction,
            &[
                name_service_program.clone(),
                fee_payer.clone(),
                name_account.clone(),
                new_owner_account.clone(),
                system_program_account.clone(),
                root_name_account.clone(),
                authority.clone(),
            ],
            &[signer_seeds],
        )
    }
}

pub fn check_account_key(account: &AccountInfo, key: &Pubkey) -> ProgramResult {
    if account.key != key {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_account_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    if !(account.is_signer) {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

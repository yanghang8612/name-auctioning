use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_auction::{
    instruction::{claim_bid_instruction, create_auction_instruction, start_auction_instruction},
    processor::{ClaimBidArgs, CreateAuctionArgs, PriceFloor, StartAuctionArgs, WinnerLimit},
};
use spl_name_service::{instruction::NameRegistryInstruction, state::NameRecordHeader};

use crate::{
    processor::{END_AUCTION_GAP, TOKEN_MINT},
    state::{NameAuction, ReverseLookup},
};

use unicode_segmentation::UnicodeSegmentation;

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
        buy_now_account: Option<&AccountInfo<'a>>,
        resource: Pubkey,
        minimum_price: u64,
        signer_seeds: &[&[u8]],
        max_price: Option<u64>,
    ) -> ProgramResult {
        let buy_now_pubkey: Option<Pubkey> = buy_now_account.map(|account| *account.key);

        let create_auction_instruction = create_auction_instruction(
            *auction_program.key,
            *fee_payer.key,
            *authority.key,
            buy_now_pubkey,
            CreateAuctionArgs {
                winners: WinnerLimit::Capped(1),
                end_auction_at: end_auction_at.map(|n| n as i64),
                end_auction_gap: Some(END_AUCTION_GAP as i64),
                token_mint: TOKEN_MINT,
                resource,
                price_floor: PriceFloor::MinimumPrice([minimum_price, 0, 0, 0]),
                max_price,
            },
        );

        let mut account_infos = vec![
            auction_program.clone(),
            fee_payer.clone(),
            auction_account.clone(),
            rent_sysvar_account.clone(),
            system_account.clone(),
            authority.clone(),
        ];

        if let Some(a) = buy_now_account {
            account_infos.push(a.clone())
        }

        invoke_signed(&create_auction_instruction, &account_infos, &[signer_seeds])
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
        destination_token_account: &AccountInfo<'a>,
        winner_account: &AccountInfo<'a>,
        winner_pot_account: &AccountInfo<'a>,
        winner_pot_token_account: &AccountInfo<'a>,
        quote_mint: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        bonfida_vault: &AccountInfo<'a>,
        buy_now: &AccountInfo<'a>,
        bonfida_sol_vault: &AccountInfo<'a>,
        referrer: Option<&AccountInfo<'a>>,
        resource: Pubkey,
        signer_seeds: &[&[u8]],
        fee_percentage: u64,
    ) -> ProgramResult {
        let claim_auction_instruction = claim_bid_instruction(
            *auction_program.key,
            *destination_token_account.key,
            *authority.key,
            *winner_account.key,
            *winner_pot_token_account.key,
            *quote_mint.key,
            *bonfida_vault.key,
            *buy_now.key,
            *bonfida_sol_vault.key,
            referrer.map(|acc| *acc.key),
            ClaimBidArgs {
                resource,
                fee_percentage,
            },
        );

        let mut accounts = vec![
            auction_program.clone(),
            destination_token_account.clone(),
            winner_pot_token_account.clone(),
            winner_pot_account.clone(),
            authority.clone(),
            auction_account.clone(),
            winner_account.clone(),
            quote_mint.clone(),
            clock_sysvar_account.clone(),
            spl_token_program.clone(),
            bonfida_vault.clone(),
            buy_now.clone(),
            bonfida_sol_vault.clone(),
        ];

        if let Some(referrer) = referrer {
            accounts.push(referrer.clone());
        }

        invoke_signed(&claim_auction_instruction, &accounts, &[signer_seeds])
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

    #[allow(clippy::too_many_arguments)]
    pub fn create_reverse_lookup_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        reverse_lookup_account: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        name: String,
        hashed_reverse_lookup: Vec<u8>,
        authority: &AccountInfo<'a>,
        rent_sysvar_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        parent_name: Option<&AccountInfo<'a>>,
        parent_name_opt: Option<&AccountInfo<'a>>,
    ) -> ProgramResult {
        let name_bytes = ReverseLookup { name }.try_to_vec().unwrap();
        let rent = Rent::from_account_info(rent_sysvar_account)?;
        let lamports = rent.minimum_balance(name_bytes.len() + NameRecordHeader::LEN);

        let create_name_instruction = spl_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name: hashed_reverse_lookup,
                lamports,
                space: name_bytes.len() as u32,
            },
            *reverse_lookup_account.key,
            *fee_payer.key,
            *authority.key,
            Some(*authority.key),
            parent_name.map(|a| *a.key),
            parent_name_opt.map(|a| *a.key),
        )?;

        let mut accounts_create = vec![
            name_service_program.clone(),
            fee_payer.clone(),
            authority.clone(),
            reverse_lookup_account.clone(),
            system_program_account.clone(),
        ];

        let mut accounts_update = vec![
            name_service_program.clone(),
            reverse_lookup_account.clone(),
            authority.clone(),
        ];

        if let Some(parent_name) = parent_name {
            accounts_create.push(parent_name.clone());
            accounts_update.push(parent_name.clone())
        }

        invoke_signed(&create_name_instruction, &accounts_create, &[signer_seeds])?;

        let write_name_instruction = spl_name_service::instruction::update(
            *name_service_program.key,
            0,
            name_bytes,
            *reverse_lookup_account.key,
            *authority.key,
            parent_name.map(|a| *a.key),
        )?;

        invoke_signed(&write_name_instruction, &accounts_update, &[signer_seeds])?;
        Ok(())
    }

    pub fn transfer_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        old_owner_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        new_owner_key: &Pubkey,
        signer_seeds: Option<&[&[u8]]>,
    ) -> ProgramResult {
        let transfer_name_instruction = spl_name_service::instruction::transfer(
            *name_service_program.key,
            *new_owner_key,
            *name_account.key,
            *old_owner_account.key,
            None,
        )?;

        if let Some(seeds) = signer_seeds {
            invoke_signed(
                &transfer_name_instruction,
                &[
                    name_service_program.clone(),
                    old_owner_account.clone(),
                    name_account.clone(),
                ],
                &[seeds],
            )
        } else {
            invoke(
                &transfer_name_instruction,
                &[
                    name_service_program.clone(),
                    old_owner_account.clone(),
                    name_account.clone(),
                ],
            )
        }
    }

    pub fn end_auction<'a>(
        auction_program: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        auction: &AccountInfo<'a>,
        clock_sysvar_account: &AccountInfo<'a>,
        resource: Pubkey,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let end_auction_instruction = spl_auction::instruction::end_auction_instruction(
            *auction_program.key,
            *authority.key,
            spl_auction::processor::EndAuctionArgs {
                resource,
                reveal: None,
            },
        );

        invoke_signed(
            &end_auction_instruction,
            &[
                auction_program.clone(),
                authority.clone(),
                auction.clone(),
                clock_sysvar_account.clone(),
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

pub fn get_usd_price(len: usize) -> u64 {
    let multiplier = match len {
        1 => 750,
        2 => 700,
        3 => 640,
        4 => 160,
        _ => 20,
    };
    multiplier * 1_000_000
}

pub fn get_grapheme_len(name: &String) -> usize {
    name.graphemes(true).count()
}

#[test]
pub fn test_length() {
    let string_1 = "1".to_string();
    let string_2 = "12".to_string();
    let string_3 = "jkfdgnjkdfgn".to_string();
    let string_4 = "😀".to_string();
    let string_5 = "◎x".to_string();
    let string_6 = "🏳️‍🌈".to_string();

    assert_eq!(get_grapheme_len(&string_1), 1);
    assert_eq!(get_grapheme_len(&string_2), 2);
    assert_eq!(get_grapheme_len(&string_3), 12);
    assert_eq!(get_grapheme_len(&string_4), 1);
    assert_eq!(get_grapheme_len(&string_5), 2);
    assert_eq!(get_grapheme_len(&string_6), 1);
}

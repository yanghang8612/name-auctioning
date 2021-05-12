use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::instruction::{
    initialize_account, initialize_mint, mint_to, set_authority, AuthorityType,
};

use crate::state::TokenAuthority;

pub struct Cpi {}

impl Cpi {
    pub fn initialize_mint<'a>(
        spl_token_program: &AccountInfo<'a>,
        mint: &AccountInfo<'a>,
        rent_sysvar: &AccountInfo<'a>,
        mint_authority: &AccountInfo<'a>,
    ) -> ProgramResult {
        let instruction = initialize_mint(&spl_token::id(), mint.key, mint_authority.key, None, 0)?;
        invoke(
            &instruction,
            &[spl_token_program.clone(), mint.clone(), rent_sysvar.clone()],
        )
    }

    pub fn initialize_token_account<'a>(
        spl_token_program: &AccountInfo<'a>,
        rent_sysvar: &AccountInfo<'a>,
        target_token_account: &AccountInfo<'a>,
        target_token_account_owner: &AccountInfo<'a>,
        token_mint: &AccountInfo<'a>,
    ) -> ProgramResult {
        let instruction = initialize_account(
            &spl_token::id(),
            target_token_account.key,
            token_mint.key,
            target_token_account_owner.key,
        )?;
        invoke(
            &instruction,
            &[
                spl_token_program.clone(),
                target_token_account.clone(),
                token_mint.clone(),
                target_token_account_owner.clone(),
                rent_sysvar.clone(),
            ],
        )
    }

    pub fn mint_nft<'a>(
        spl_token_program: &AccountInfo<'a>,
        token_mint: &AccountInfo<'a>,
        target_token_account: &AccountInfo<'a>,
        signer_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let instruction = mint_to(
            &spl_token::id(),
            token_mint.key,
            target_token_account.key,
            signer_account.key,
            &[],
            1,
        )?;
        invoke_signed(
            &instruction,
            &[
                spl_token_program.clone(),
                token_mint.clone(),
                target_token_account.clone(),
                signer_account.clone(),
            ],
            &[signer_seeds],
        )
    }

    pub fn disable_mint<'a>(
        spl_token_program: &AccountInfo<'a>,
        token_mint: &AccountInfo<'a>,
        signer_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let instruction = set_authority(
            &spl_token::id(),
            token_mint.key,
            None,
            AuthorityType::MintTokens,
            signer_account.key,
            &[],
        )?;

        invoke_signed(
            &instruction,
            &[
                spl_token_program.clone(),
                token_mint.clone(),
                signer_account.clone(),
            ],
            &[signer_seeds],
        )
    }

    pub fn create_account<'a>(
        program_id: &Pubkey,
        system_program: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        signer_account: &AccountInfo<'a>,
        rent_sysvar_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let rent = Rent::from_account_info(rent_sysvar_account)?;

        let create_state_instruction = create_account(
            fee_payer.key,
            signer_account.key,
            rent.minimum_balance(TokenAuthority::LEN),
            TokenAuthority::LEN as u64,
            program_id,
        );

        invoke_signed(
            &create_state_instruction,
            &[
                system_program.clone(),
                fee_payer.clone(),
                signer_account.clone(),
            ],
            &[signer_seeds],
        )
    }
}

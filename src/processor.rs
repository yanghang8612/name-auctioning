use std::{convert::TryInto};

use crate::{instructions::TokenizationInstruction, state::TokenAuthority, utils::Cpi};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;
pub struct Processor {}

impl Processor {
    fn process_create(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        signer_nonce: u8,
        seeds: &[u8],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let rent_sysvar_account = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let spl_token_program = next_account_info(accounts_iter)?;
        let signer_account = next_account_info(accounts_iter)?;
        let parent_authority_account = next_account_info(accounts_iter)?;
        let fee_payer = next_account_info(accounts_iter)?;
        let token_mint = next_account_info(accounts_iter)?;
        let target_token_account = next_account_info(accounts_iter)?;
        let target_token_account_owner = next_account_info(accounts_iter)?;

        if !parent_authority_account.is_signer {
            msg!("The initializer account must sign for the creation of this new authority");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if seeds.len() != 32 {
            msg!("Invalid seed length");
            return Err(ProgramError::InvalidArgument);
        }

        let signer_seeds = [&parent_authority_account.key.to_bytes(), seeds, &[signer_nonce]];

        let derived_signer_key = Pubkey::create_program_address(&signer_seeds, program_id)
            .map_err(|_| {
                msg!("Invalid signer nonce provided.");
                ProgramError::InvalidSeeds
            })?;
        if &derived_signer_key != signer_account.key {
            msg!("An invalid signer account was provided");
            return Err(ProgramError::InvalidArgument);
        }

        Cpi::create_account(
            program_id,
            system_program,
            fee_payer,
            signer_account,
            rent_sysvar_account,
            &signer_seeds,
        )?;

        Cpi::initialize_mint(
            spl_token_program,
            token_mint,
            rent_sysvar_account,
            signer_account,
        )?;
        Cpi::initialize_token_account(
            spl_token_program,
            rent_sysvar_account,
            target_token_account,
            target_token_account_owner,
            token_mint,
        )?;
        Cpi::mint_nft(
            spl_token_program,
            token_mint,
            target_token_account,
            signer_account,
            &signer_seeds,
        )?;
        Cpi::disable_mint(spl_token_program, token_mint, signer_account, &signer_seeds)?;

        let state = TokenAuthority {
            is_initialized: true,
            mint: token_mint.key.to_bytes(),
            signer_nonce,
            seeds: seeds.try_into().unwrap(),
            parent_authority_account: parent_authority_account.key.to_bytes(),
        };

        let mut pt: &mut [u8] = &mut signer_account.data.borrow_mut();
        state.serialize(&mut pt)?;
        Ok(())
    }

    fn process_execute_instruction(
        accounts: &[AccountInfo],
        instruction_data: Vec<u8>,
    ) -> ProgramResult {
        let total_accounts_length = accounts.len();
        let accounts_iter = &mut accounts.iter();
        let token_account = next_account_info(accounts_iter)?;
        let token_account_owner = next_account_info(accounts_iter)?;
        let signer_account = next_account_info(accounts_iter)?;
        let target_program_account = next_account_info(accounts_iter)?;

        if token_account.owner != &spl_token::id() {
            msg!("The token account should be an SPL token account");
            return Err(ProgramError::InvalidArgument);
        }
        
        let signer_state = {
            let mut buf: &[u8] = &signer_account.data.borrow();
            TokenAuthority::deserialize(&mut buf)?
        };
        let signer_mint = Pubkey::new(&signer_state.mint);
        let token_account_state = {
            let d = &token_account.data.borrow();
            Account::unpack_from_slice(d)?
        };

        if signer_mint != token_account_state.mint {
            msg!("The provided token account's mint address does not match the required signer");
            return Err(ProgramError::InvalidArgument);
        }

        if token_account_state.amount != 1 {
            msg!("The provided token account doesn't own the authority NFT");
            return Err(ProgramError::InvalidArgument);
        }

        if &token_account_state.owner != token_account_owner.key {
            msg!("Invalid token account owner");
            return Err(ProgramError::InvalidArgument);
        }

        if !token_account_owner.is_signer {
            msg!("The token account owner should sign the transaction");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let signer_seeds: &[&[u8]] = &[
            &signer_state.parent_authority_account,
            &signer_state.seeds,
            &[signer_state.signer_nonce],
        ];

        let mut account_metas = Vec::with_capacity(total_accounts_length - 4);

        for a in accounts[4..].iter() {
            let is_signer = a.is_signer | (a.key == signer_account.key);
            account_metas.push(AccountMeta {
                pubkey: *a.key,
                is_signer,
                is_writable: a.is_writable,
            });
        }

        let instruction = Instruction {
            program_id: *target_program_account.key,
            accounts: account_metas,
            data: instruction_data,
        };

        invoke_signed(&instruction, &accounts[4..], &[signer_seeds])?;

        Ok(())
    }

    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = TokenizationInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        msg!("Instruction unpacked");

        match instruction {
            TokenizationInstruction::Create {
                seeds,
                signer_nonce,
            } => {
                msg!("Instruction: Create");
                Processor::process_create(program_id, accounts, signer_nonce, &seeds)?;
            }
            TokenizationInstruction::Execute { instruction_data } => {
                msg!("Instruction: Execute");
                Processor::process_execute_instruction(accounts, instruction_data)?;
            }
        }

        Ok(())
    }
}

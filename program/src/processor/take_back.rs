use crate::processor::{CENTRAL_STATE, ROOT_DOMAIN_ACCOUNT};
use crate::utils::{check_account_key, check_account_owner, check_signer};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
};
use spl_name_service::instruction::NameRegistryInstruction;

fn transfer(
    name_service_program_id: Pubkey,
    new_owner: Pubkey,
    name_account_key: Pubkey,
    name_owner_key: Pubkey,
    name_class_opt: Option<Pubkey>,
    name_parent: Option<Pubkey>,
) -> Result<Instruction, ProgramError> {
    let instruction_data = NameRegistryInstruction::Transfer { new_owner };
    let data = instruction_data.try_to_vec().unwrap();
    let mut accounts = vec![
        AccountMeta::new(name_account_key, false),
        AccountMeta::new_readonly(name_owner_key, true),
    ];

    if let Some(key) = name_class_opt {
        accounts.push(AccountMeta::new_readonly(key, false));
    }

    if let Some(key) = name_parent {
        accounts.push(AccountMeta::new_readonly(key, false));
    }

    Ok(Instruction {
        program_id: name_service_program_id,
        accounts,
        data,
    })
}

const ADMIN: Pubkey = pubkey!("VBx642K1hYGLU5Zm1CHW1uRXAtFgxN5mRqyMcXnLZFW");

struct Accounts<'a, 'b: 'a> {
    admin: &'a AccountInfo<'b>,
    naming_service_program: &'a AccountInfo<'b>,
    name: &'a AccountInfo<'b>,
    name_owner: &'a AccountInfo<'b>,
    name_class: &'a AccountInfo<'b>,
    parent_name: &'a AccountInfo<'b>,
    new_owner: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let a = Accounts {
        admin: next_account_info(accounts_iter)?,
        naming_service_program: next_account_info(accounts_iter)?,
        name: next_account_info(accounts_iter)?,
        name_owner: next_account_info(accounts_iter)?,
        name_class: next_account_info(accounts_iter)?,
        parent_name: next_account_info(accounts_iter)?,
        new_owner: next_account_info(accounts_iter)?,
    };

    // Check keys
    check_account_key(a.admin, &ADMIN).unwrap();
    check_account_key(a.naming_service_program, &spl_name_service::id()).unwrap();
    check_account_key(a.name_owner, &CENTRAL_STATE).unwrap();

    // Check ownership
    check_account_owner(a.name, &spl_name_service::ID).unwrap();

    // Check signer
    check_signer(a.admin).unwrap();

    Ok(a)
}

pub fn process_take_back(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // let accounts = parse_accounts(accounts)?;
    // let central_state_nonce = accounts.name_owner.data.borrow()[0];

    // let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state_nonce]];

    // let ix = transfer(
    //     spl_name_service::ID,
    //     *accounts.new_owner.key,
    //     *accounts.name.key,
    //     *accounts.name_owner.key,
    //     Some(Pubkey::default()),
    //     Some(ROOT_DOMAIN_ACCOUNT),
    // )
    // .unwrap();

    // invoke_signed(
    //     &ix,
    //     &[
    //         accounts.naming_service_program.clone(),
    //         accounts.name.clone(),
    //         accounts.name_owner.clone(),
    //         accounts.name_class.clone(),
    //         accounts.parent_name.clone(),
    //     ],
    //     &[central_state_signer_seeds],
    // )?;

    Ok(())
}

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    sysvar::rent,
};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum ProgramInstruction {
    /// Creates an auction
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The system program account
    ///   3. `[]` The SPL token program account
    ///   4. `[writable]` The new signer account
    ///   5. `[signer]` The parent authority account
    ///   6. `[signer, writable]` The fee payer account
    ///   7. `[writable]` The new nft token mint account
    ///   8. `[writable]` The nft token account to be created and minted to
    ///   9. `[]` The owner of the target nft token account
    Init { state_nonce: u8 },
    /// Creates an auction
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The rent sysvar account
    ///   2. `[]` The system program account
    ///   3. `[]` The SPL token program account
    ///   4. `[writable]` The new signer account
    ///   5. `[signer]` The parent authority account
    ///   6. `[signer, writable]` The fee payer account
    ///   7. `[writable]` The new nft token mint account
    ///   8. `[writable]` The nft token account to be created and minted to
    ///   9. `[]` The owner of the target nft token account
    Create { hashed_name: [u8; 32] },
    /// Executes an arbitrary program instruction, signing as the tokenized authority
    ///
    /// Accounts expected by this instruction:
    ///
    ///   1. `[]` The nft token account
    ///   2. `[signer]` The owner of the nft token account
    ///   3. `[]` The associateed signer account
    ///   3. `[]` The target program account
    ///   4... `[?]` The necessary accounts for the instruction, in instruction order.
    ///               All instances of the signer account will be set as signer when calling the instruction
    Claim {
        hashed_name: Vec<u8>,
        lamports: u64,
        space: u32,
    },
}

pub struct TokenAuthorityContext {
    pub program_id: Pubkey,
    pub signer_key: Pubkey,
    pub mint: Pubkey,
}

pub fn create(
    ctx: &TokenAuthorityContext,
    parent_authority: Pubkey,
    fee_payer: Pubkey,
    target_token_account: Pubkey,
    target_token_account_owner: Pubkey,
    seeds: [u8; 32],
) -> Instruction {
    let data = ProgramInstruction::Create { hashed_name: seeds }
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(ctx.signer_key, false),
        AccountMeta::new_readonly(parent_authority, true),
        AccountMeta::new(fee_payer, true),
        AccountMeta::new(ctx.mint, false),
        AccountMeta::new(target_token_account, false),
        AccountMeta::new(target_token_account_owner, false),
    ];
    Instruction {
        program_id: ctx.program_id,
        accounts,
        data,
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::{Pack, Sealed}, program_error::ProgramError};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenAuthority {
    pub is_initialized: bool,
    pub mint: [u8; 32],
    pub seeds: [u8; 32],
    pub parent_authority_account: [u8; 32],
    pub signer_nonce: u8,
}

impl Sealed for TokenAuthority {}

impl Pack for TokenAuthority {
    const LEN: usize = 98;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut pt = dst;
        self.serialize(&mut pt).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut pt = src;
        let res = Self::deserialize(&mut pt)?;
        Ok(res)
    }
}
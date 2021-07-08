use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
};

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
pub enum NameAuctionStatus {
    Uninitialized,
    FirstAuction,
    Claimed,
    SecondaryAuction,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NameAuction {
    pub status: NameAuctionStatus,
    pub quote_mint: [u8; 32],
    pub signer_nonce: u8,
    pub auction_account: [u8; 32],
}

impl Sealed for NameAuction {}

impl Pack for NameAuction {
    const LEN: usize = 66;

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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ResellingAuction {
    pub token_destination_account: [u8; 32],
}

impl Sealed for ResellingAuction {}

impl Pack for ResellingAuction {
    const LEN: usize = 32;

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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CentralState {
    pub signer_nonce: u8,
}

impl Sealed for CentralState {}

impl Pack for CentralState {
    const LEN: usize = 1;

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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ReverseLookup {
    pub name: String,
}

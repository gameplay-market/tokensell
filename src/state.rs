use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
  pubkey::Pubkey,
  clock::UnixTimestamp,
  account_info::AccountInfo,
  program_error::ProgramError,
  borsh::try_from_slice_unchecked
};

use crate::{
    error::TokensellError
};

pub static PARTICIPANT_SIZE: usize = 1 + 32 + 32 + 32 + 8;
pub static TOKENSELL_SELL_SIZE: usize = 1 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 8;

pub static PREFIX: &str = "tokensell";

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Key {
    Uninitialized,
    ParticipantData,
    SellData,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ParticipantData {
    pub key: Key,
    pub owner: Pubkey,
    pub sell: Pubkey,
    pub amount: u64,
    pub claimed: u64,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SellData {
    pub key: Key,
    pub owner: Pubkey,
    pub source_mint: Pubkey,
    pub target_mint: Option<Pubkey>,
    pub source_vault: Option<Pubkey>,
    pub target_acc: Pubkey,
    pub exchange_rate: u64,
    pub start_time: UnixTimestamp,
    pub end_time: UnixTimestamp,
    pub tge: Option<UnixTimestamp>,
    pub initial_perc: u64,
    pub total_months: u64,
    pub min_deposit: u64,
    pub amount_total: u64,
    pub amount_left: u64,
}

impl SellData {
    pub fn from_account_info(a: &AccountInfo) -> Result<SellData, ProgramError> {
        if a.data_len() < TOKENSELL_SELL_SIZE {
            return Err(TokensellError::SizeMismatch.into());
        }

        let auction: SellData = try_from_slice_unchecked(&a.data.borrow_mut())?;

        Ok(auction)
    }
}

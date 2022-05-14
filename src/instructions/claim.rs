use solana_program::{
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  account_info::{next_account_info, AccountInfo},
  msg,
  program_pack::Pack,
  sysvar::{clock::Clock, Sysvar},
};

use spl_token::{
  state::{Mint}
};

use borsh::{BorshDeserialize};

use crate::{
  state::{
    Key,
    ParticipantData,
    SellData,
    PREFIX,
  },
  error::{TokensellError},
  utils::{
    spl_token_transfer,
    TokenTransferParams,
  }
};

static MONTH_SECONDS: u64 = 30 * 24 * 3600;

pub fn process_claim(program_id: &Pubkey, accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();

  let payer_info = next_account_info(account_info_iter)?;
  let sell_info = next_account_info(account_info_iter)?;
  let target_mint_info = next_account_info(account_info_iter)?;
  let sell_authority_info = next_account_info(account_info_iter)?;
  let sell_vault_info = next_account_info(account_info_iter)?;
  let token_program_info = next_account_info(account_info_iter)?;
  let participant_info = next_account_info(account_info_iter)?;
  let target_token_info = next_account_info(account_info_iter)?;
  let clock_sysvar_info = next_account_info(account_info_iter)?;

  if !payer_info.is_signer {
    return Err(TokensellError::SignatureRequired.into());
  }

  if *sell_info.owner != *program_id {
    msg!("Invalid sell account owner");
    return Err(TokensellError::InvalidOwner.into());
  }
  
  let sell = SellData::from_account_info(sell_info)?;

  if sell.key != Key::SellData {
    return Err(TokensellError::InvalidAccount.into());
  }

  if sell.tge.is_none() || sell.source_vault.is_none() || sell.target_mint.is_none() {
    return Err(TokensellError::NothingToClaim.into());
  }

  if *target_mint_info.key != sell.target_mint.unwrap() {
    return Err(TokensellError::InvalidAccount.into());
  }

  let participant_key = Pubkey::find_program_address(
    &[
      PREFIX.as_bytes(),
      program_id.as_ref(),
      sell_info.key.as_ref(),
      payer_info.key.as_ref()
    ],
    program_id
  ).0;

  if participant_key != *participant_info.key {
    return Err(TokensellError::InvalidAccount.into());
  }

  if sell.source_vault.unwrap() != *sell_vault_info.key {
    return Err(TokensellError::InvalidAccount.into());
  }
  
  let target_mint = Mint::unpack(&target_mint_info.data.borrow_mut())?;
  let participant = ParticipantData::try_from_slice(&participant_info.data.borrow_mut())?;
  let clock = Clock::from_account_info(&clock_sysvar_info)?;

  let amount;
  if clock.unix_timestamp <= sell.tge.unwrap() {
    amount = 0;
  } else {
    let months_passed = (clock.unix_timestamp - sell.tge.unwrap()) as u64 / MONTH_SECONDS;

    msg!("Claim {} {}", months_passed, sell.total_months);

    let total_amount = participant.amount;
    let to_claim = total_amount - participant.claimed;

    if months_passed < sell.total_months {
      let initial_amount = total_amount * sell.initial_perc / 100;
      let month_amount = (total_amount - initial_amount) * months_passed / sell.total_months;
      let freezed_amount = (total_amount - initial_amount - month_amount) * 10u64.pow(target_mint.decimals.into());
  
      msg!(
        "Claim2 {}, {}, {}, {}",
        total_amount,
        initial_amount,
        month_amount,
        freezed_amount
      );
  
      if to_claim > freezed_amount {
        amount = to_claim - freezed_amount;
      } else {
        amount = 0;
      }
    } else {
      amount = to_claim;
    }
    
    msg!(
      "Claim3 {}",
      amount
    );
  }

  if amount == 0 {
    return Err(TokensellError::NothingToClaim.into());
  }

  let (sell_authority, sell_bump) = Pubkey::find_program_address(&[
    PREFIX.as_bytes(),
    program_id.as_ref(),
    sell_info.key.as_ref(),
  ], program_id);

  if sell_authority != *sell_authority_info.key {
    return Err(TokensellError::InvalidAccount.into());
  }

  spl_token_transfer(TokenTransferParams {
    source: sell_vault_info.clone(),
    destination: target_token_info.clone(),
    amount: amount,
    authority: sell_authority_info.clone(),
    authority_signer_seeds: &[
      PREFIX.as_bytes(),
      program_id.as_ref(),
      sell_info.key.as_ref(),
      &[sell_bump]
    ],
    token_program: token_program_info.clone(),
  })?;
  
  Ok(())
}

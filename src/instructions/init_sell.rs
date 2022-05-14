use solana_program::{
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  account_info::{next_account_info, AccountInfo},
  msg,
  program_pack::Pack,
  clock::UnixTimestamp,
  sysvar::{
    clock::Clock,
    rent::Rent,
    Sysvar
  }
};

use borsh::{BorshSerialize, BorshDeserialize};

use spl_token::{
  state::Account,
};

use crate::{
  state::{Key, SellData, PREFIX},
  error::{TokensellError},
  utils::{assert_rent_exempt}
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct InitSellArgs {
  pub instruction: u8,
  pub exchange_rate: u64,
  pub start_time: UnixTimestamp,
  pub end_time: UnixTimestamp,
  pub initial_perc: u64,
  pub total_months: u64,
  pub min_deposit: u64,
  pub total_amount: u64,
}

pub fn process_init_sell(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
  let args = InitSellArgs::try_from_slice(instruction_data)?;

  let account_info_iter = &mut accounts.iter();

  let payer_info = next_account_info(account_info_iter)?;
  let sell_info = next_account_info(account_info_iter)?;
  let token_vault_info = next_account_info(account_info_iter)?;
  let target_mint_info = next_account_info(account_info_iter)?;
  let source_mint_info = next_account_info(account_info_iter)?;
  let target_info = next_account_info(account_info_iter)?;
  let rent_sysvar_info = next_account_info(account_info_iter)?;
  let clock_sysvar_info = next_account_info(account_info_iter)?;

  if *sell_info.owner != *program_id {
    return Err(TokensellError::InvalidOwner.into());
  }

  let rent = &Rent::from_account_info(rent_sysvar_info)?;
  let clock = &Clock::from_account_info(clock_sysvar_info)?;

  let mut sell = SellData::from_account_info(sell_info)?;
  let token_vault = Account::unpack_from_slice(&token_vault_info.data.borrow_mut())?;
  let target_acc = Account::unpack_from_slice(&target_info.data.borrow_mut())?;

  assert_rent_exempt(rent, sell_info)?;

  if sell.key != Key::Uninitialized {
    return Err(TokensellError::AccountInitialized.into());
  }

  msg!("Init sell {} {} {}", clock.unix_timestamp, args.start_time, args.end_time);
  
  if clock.unix_timestamp > args.end_time {
    msg!("End time cant be less than current time");
    return Err(TokensellError::InvalidEndTimestamp.into());
  }
  
  if args.end_time < args.start_time {
    msg!("End time cant be less than start time");
    return Err(TokensellError::InvalidEndTimestamp.into());
  }
  
  let (sell_authority, _bump) = Pubkey::find_program_address(&[
    PREFIX.as_bytes(),
    program_id.as_ref(),
    sell_info.key.as_ref(),
  ], program_id);

  msg!("Vault balance {}", token_vault.amount);

  if token_vault.owner != sell_authority {
    msg!("Invalid vault owner {}", sell_authority);
    return Err(TokensellError::InvalidOwner.into());
  }

  if token_vault.amount <= 0 {
    return Err(TokensellError::NoTokensInVault.into());
  }

  if token_vault.mint != *target_mint_info.key {
    return Err(TokensellError::WrongMint.into());
  }

  if target_acc.mint != *source_mint_info.key {
    return Err(TokensellError::WrongMint.into());
  }

  sell.key = Key::SellData;
  sell.owner = *payer_info.key;
  sell.source_mint = *source_mint_info.key;
  sell.target_mint = None;
  sell.source_vault = None;
  sell.target_acc = *target_info.key;
  sell.exchange_rate = args.exchange_rate;
  sell.start_time = args.start_time;
  sell.end_time = args.end_time;
  sell.initial_perc = args.initial_perc;
  sell.total_months = args.total_months;
  sell.min_deposit = args.min_deposit;
  sell.amount_total = args.total_amount;
  sell.amount_left = args.total_amount;
  sell.tge = None;
  
  sell.serialize(&mut *sell_info.data.borrow_mut())?;

  Ok(())
}
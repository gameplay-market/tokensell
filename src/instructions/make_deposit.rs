use solana_program::{
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  account_info::{next_account_info, AccountInfo},
  msg,
  sysvar::{clock::Clock, Sysvar}
};

use borsh::{BorshSerialize, BorshDeserialize};

use crate::{
  state::{
    Key,
    ParticipantData,
    SellData,
    PARTICIPANT_SIZE,
    PREFIX,
  },
  error::{TokensellError},
  utils::{
    create_or_allocate_account_raw,
    spl_token_transfer,
    TokenTransferParams,
  }
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct MakeDepositArgs {
  pub instruction: u8,
  pub amount: u64,
}

pub fn process_make_deposit(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
  let args = MakeDepositArgs::try_from_slice(instruction_data)?;

  let account_info_iter = &mut accounts.iter();

  let payer_info = next_account_info(account_info_iter)?;
  let usdt_source_info = next_account_info(account_info_iter)?;
  let sell_info = next_account_info(account_info_iter)?;
  let usdt_target_info = next_account_info(account_info_iter)?;
  let transfer_authority_info = next_account_info(account_info_iter)?;
  let token_program_info = next_account_info(account_info_iter)?;
  let participant_info = next_account_info(account_info_iter)?;
  let rent_sysvar_info = next_account_info(account_info_iter)?;
  let clock_sysvar_info = next_account_info(account_info_iter)?;
  let system_program_info = next_account_info(account_info_iter)?;

  if !payer_info.is_signer {
    return Err(TokensellError::SignatureRequired.into());
  }

  if *token_program_info.key != spl_token::id() {
    msg!("Invalid token program");
    return Err(TokensellError::InvalidAccount.into());
  }

  if *sell_info.owner != *program_id {
    msg!("Invalid sell account owner");
    return Err(TokensellError::InvalidOwner.into());
  }

  let sell = SellData::from_account_info(sell_info)?;

  if sell.key != Key::SellData {
    return Err(TokensellError::InvalidAccount.into());
  }

  let clock = Clock::from_account_info(&clock_sysvar_info)?;

  if clock.unix_timestamp < sell.start_time {
    return Err(TokensellError::SellNotStarted.into());
  }

  if clock.unix_timestamp > sell.end_time {
    return Err(TokensellError::SellEnded.into());
  }
  
  if *usdt_target_info.key != sell.target_acc {
    msg!("Invalid target token account {} {}", usdt_target_info.key, sell.target_acc);
    return Err(TokensellError::InvalidAccount.into());
  }

  let usdt_amount = sell.exchange_rate * args.amount;

  let (participant_key, bump) = Pubkey::find_program_address(
    &[
      PREFIX.as_bytes(),
      program_id.as_ref(),
      sell_info.key.as_ref(),
      payer_info.key.as_ref()
    ],
    program_id
  );

  if participant_key != *participant_info.key {    
    msg!("Invalid participant account");
    return Err(TokensellError::InvalidAccount.into());
  }

  if participant_info.data_is_empty() {
    if usdt_amount < sell.min_deposit {
      return Err(TokensellError::MinimalDeposit.into());
    }

    msg!("Create account");

    create_or_allocate_account_raw(
      *program_id,
      participant_info,
      rent_sysvar_info,
      system_program_info,
      payer_info,
      PARTICIPANT_SIZE,
      &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        sell_info.key.as_ref(),
        payer_info.key.as_ref(),
        &[bump]
      ]
    )?;

    ParticipantData {
      key: Key::ParticipantData,
      owner: *payer_info.key,
      sell: *sell_info.key,
      amount: args.amount,
      claimed: 0,
    }.serialize(&mut *participant_info.data.borrow_mut())?;
  } else {
    let mut data = ParticipantData::try_from_slice(&participant_info.data.borrow_mut())?;
  
    data.amount = data.amount
      .checked_add(args.amount)
      .ok_or(TokensellError::Overflow)?;

    data.serialize(&mut *participant_info.data.borrow_mut())?;
  }

  msg!("Start transfer {}", usdt_amount);

  spl_token_transfer(TokenTransferParams {
    source: usdt_source_info.clone(),
    destination: usdt_target_info.clone(),
    amount: usdt_amount,
    authority: transfer_authority_info.clone(),
    authority_signer_seeds: &[],
    token_program: token_program_info.clone(),
  })?;

  msg!("Hello from {}", program_id);

  Ok(())
}
use solana_program::{
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  account_info::{next_account_info, AccountInfo},
  msg,
  program_pack::Pack,
  clock::UnixTimestamp,
};

use spl_token::{
  state::Account,
};

use borsh::{BorshSerialize, BorshDeserialize};

use crate::{
  state::{Key, SellData, PREFIX},
  error::{TokensellError},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SetTgeArgs {
  pub instruction: u8,
  pub tge: Option<UnixTimestamp>,
}

pub fn process_set_tge(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
  let args = SetTgeArgs::try_from_slice(instruction_data)?;

  let account_info_iter = &mut accounts.iter();

  let payer_info = next_account_info(account_info_iter)?;
  let sell_info = next_account_info(account_info_iter)?;
  let target_mint_info = next_account_info(account_info_iter)?;
  let source_vault_info = next_account_info(account_info_iter)?;
  
  if !payer_info.is_signer {
    return Err(TokensellError::SignatureRequired.into());
  }

  if *sell_info.owner != *program_id {
    return Err(TokensellError::InvalidOwner.into());
  }

  let mut sell = SellData::from_account_info(sell_info)?;
  let token_vault = Account::unpack_from_slice(&source_vault_info.data.borrow_mut())?;

  if sell.key != Key::SellData {
    return Err(TokensellError::InvalidAccount.into());
  }

  if sell.owner != *payer_info.key {
    return Err(TokensellError::InvalidAccount.into());
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

  if token_vault.amount < sell.amount_total {
    return Err(TokensellError::NoTokensInVault.into());
  }

  if token_vault.mint != *target_mint_info.key {
    return Err(TokensellError::WrongMint.into());
  }

  msg!("Set tge {}", args.tge.unwrap_or(-1));
  
  sell.tge = args.tge;
  
  sell.serialize(&mut *sell_info.data.borrow_mut())?;

  Ok(())
}
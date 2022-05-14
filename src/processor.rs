use solana_program::{
  entrypoint,
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  account_info::AccountInfo,
  msg,
};

use crate::{
  error::TokensellError,
  instructions::{
    make_deposit::process_make_deposit,
    init_sell::process_init_sell,
    claim::process_claim,
    set_tge::process_set_tge,
  },
};

// use borsh::{BorshSerialize, BorshDeserialize};

pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
  if instruction_data.is_empty() {
    return Err(TokensellError::UnknownInstruction.into());
  }

  let instruction = instruction_data[0];

  match instruction {
    0 => {
      msg!("Instruction: Make deposit");
      process_make_deposit(program_id, accounts, instruction_data)
    },
    1 => {
      msg!("Instruction: Init sell");
      process_init_sell(program_id, accounts, instruction_data)
    },
    2 => {
      msg!("Instruction: Claim");
      process_claim(program_id, accounts, instruction_data)
    },
    3 => {
      msg!("Instruction: Set TGE");
      process_set_tge(program_id, accounts, instruction_data)
    },
    _ => Err(TokensellError::UnknownInstruction.into())
  }
}

entrypoint!(process_instruction);


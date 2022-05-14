use solana_program::{
  entrypoint::ProgramResult,
  program::{invoke, invoke_signed},
  program_error::ProgramError,
  system_instruction,
  sysvar::{rent::Rent, Sysvar},
  pubkey::Pubkey,
  account_info::AccountInfo,
  program_pack::Pack,
  msg
};

use crate::error::TokensellError;

/// Issue a spl_token `Transfer` instruction.
#[inline(always)]
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;

    let result;
    
    if authority_signer_seeds.is_empty() {
      result = invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
      );
    } else {
      result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &[authority_signer_seeds],
      );
    }

    result.map_err(|_| TokensellError::TokenTransferFailed.into())
}


#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;
    msg!("Completed assignation!");

    Ok(())
}

/// Create a new SPL token account.
#[inline(always)]
pub fn spl_token_create_account<'a>(params: TokenCreateAccount<'_, '_>) -> ProgramResult {
    let TokenCreateAccount {
        payer,
        mint,
        account,
        authority,
        authority_seeds,
        token_program,
        system_program,
        rent,
    } = params;
    let acct = &account.key.clone();

    create_or_allocate_account_raw(
        *token_program.key,
        &account,
        &rent,
        &system_program,
        &payer,
        spl_token::state::Account::LEN,
        authority_seeds,
    )?;

    msg!("Created account {}", acct);
    
    invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            acct,
            mint.key,
            authority.key,
        )?,
        &[
            account,
            authority,
            mint,
            token_program,
            system_program,
            rent,
        ],
        &[authority_seeds],
    )?;

    Ok(())
}

pub fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Err(TokensellError::NotRentExempt.into())
    } else {
        Ok(())
    }
}

pub struct TokenTransferParams<'a: 'b, 'b> {
  /// source
  pub source: AccountInfo<'a>,
  /// destination
  pub destination: AccountInfo<'a>,
  /// amount
  pub amount: u64,
  /// authority
  pub authority: AccountInfo<'a>,
  /// authority_signer_seeds
  pub authority_signer_seeds: &'b [&'b [u8]],
  /// token_program
  pub token_program: AccountInfo<'a>,
}

pub struct TokenCreateAccount<'a: 'b, 'b> {
  /// payer
  pub payer: AccountInfo<'a>,
  /// mint
  pub mint: AccountInfo<'a>,
  /// account
  pub account: AccountInfo<'a>,
  /// authority
  pub authority: AccountInfo<'a>,
  /// authority seeds
  pub authority_seeds: &'b [&'b [u8]],
  /// token_program
  pub token_program: AccountInfo<'a>,
  pub system_program: AccountInfo<'a>,
  /// rent information
  pub rent: AccountInfo<'a>,
}

use solana_program::{msg};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;


pub fn process(
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let mut accounts = &mut accounts.iter();

    let src = next_account_info(accounts)?;
    let dst = next_account_info(accounts)?;
    let authority = next_account_info(accounts)?;

    msg!("src {:?}", src);
    msg!("dst {:?}", dst);

    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            src.key,
            dst.key,
            authority.key,
            &[],
            amount,
        )?,
        &[
            src.clone(),
            dst.clone(),
            authority.clone(),
        ],
    )?;

    Ok(())
}
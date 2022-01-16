use solana_program::{};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;


pub fn process(
    accounts: &[AccountInfo],
) -> ProgramResult {
    let mut accounts = &mut accounts.iter();

    let admin = next_account_info(accounts)?;
    let exchange_booth = next_account_info(accounts)?;

    **admin.try_borrow_mut_lamports()? = admin
        .lamports()
        .checked_add(exchange_booth.lamports())
        .ok_or(ExchangeBoothError::Overflow)?;
    **exchange_booth.try_borrow_mut_lamports()? = 0;
    *exchange_booth.try_borrow_mut_data()? = &mut [];
    Ok(())
}
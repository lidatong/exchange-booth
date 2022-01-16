use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};
use crate::processor::{deposit, withdraw};
use crate::state::Oracle;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let mut accounts = &mut accounts.iter();

    let deposit_src = next_account_info(accounts)?;
    let deposit_dst = next_account_info(accounts)?;
    let withdraw_src = next_account_info(accounts)?;
    let withdraw_dst = next_account_info(accounts)?;
    let withdraw_mint = next_account_info(accounts)?;
    let authority = next_account_info(accounts)?;
    let oracle = next_account_info(accounts)?;

    let exchange_booth = next_account_info(accounts)?;
    let exchange_booth_data: ExchangeBooth = ExchangeBooth::try_from_slice(*exchange_booth.try_borrow_data()?)?;

    if exchange_booth_data.oracle != *oracle.key {
        return Err(ExchangeBoothError::UnknownOracle.into());
    }

    let exchange_rate = Oracle::try_from_slice(&oracle.try_borrow_data()?[9..])?.exchange_rate;
    msg!("Exchange rate is {:?}", exchange_rate);
    deposit::process(&[deposit_src.clone(), deposit_dst.clone(), authority.clone()], amount)?;

    let seeds: &[&[u8]] = &[exchange_booth.key.as_ref(), withdraw_mint.key.as_ref()];
    let (vault, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let bump_seed_array: &[&[u8]] = &[&[bump_seed]];
    let seeds = [seeds, bump_seed_array].concat();

    invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            withdraw_src.key,
            withdraw_dst.key,
            &vault,
            &[],
            exchange_rate * amount,
        )?,
        &[
            withdraw_src.clone(),
            withdraw_dst.clone(),
        ],
        &[seeds.as_slice()],
    )?;

    Ok(())
}
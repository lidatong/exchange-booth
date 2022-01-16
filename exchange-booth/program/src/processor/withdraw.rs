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
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let mut accounts = &mut accounts.iter();

    let src = next_account_info(accounts)?;
    let dst = next_account_info(accounts)?;
    let mint = next_account_info(accounts)?;
    let admin = next_account_info(accounts)?;
    let exchange_booth = next_account_info(accounts)?;
    let exchange_booth_data: ExchangeBooth = ExchangeBooth::try_from_slice(*exchange_booth.try_borrow_data()?)?;

    if *admin.key != exchange_booth_data.admin {
        msg!("{:?} {:?} FAILING", admin.key, exchange_booth_data.admin);
        return Err(ProgramError::InvalidAccountData);
    }

    let seeds: &[&[u8]] = &[exchange_booth.key.as_ref(), mint.key.as_ref()];
    let (_pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let bump_seed_array: &[&[u8]] = &[&[bump_seed]];
    let seeds = [seeds, bump_seed_array].concat();

    invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            src.key,
            dst.key,
            src.key,
            &[],
            amount,
        )?,
        &[
            src.clone(),
            dst.clone(),
        ],
        &[seeds.as_slice()],
    )?;

    Ok(())
}

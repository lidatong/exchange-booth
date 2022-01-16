use std::mem::size_of;
use std::ops::DerefMut;
use solana_program::{msg, system_instruction};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts = &mut accounts.iter();

    let admin = next_account_info(accounts)?;
    msg!("ADMIN IS {:?}", admin.key);
    let mint_a = next_account_info(accounts)?;
    let mint_b = next_account_info(accounts)?;
    let vault_a = next_account_info(accounts)?;
    let vault_b = next_account_info(accounts)?;
    let oracle = next_account_info(accounts)?;
    let mut exchange_booth = next_account_info(accounts)?;
    let _system_program = next_account_info(accounts)?;
    let _spl_token = next_account_info(accounts)?;
    let rent = next_account_info(accounts)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let seeds: &[&[u8]] = &[b"exchange_booth", admin.key.as_ref(), mint_a.key.as_ref(), mint_b.key.as_ref(), oracle.key.as_ref()];
    let (_pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let bump_seed_array: &[&[u8]] = &[&[bump_seed]];
    let seeds = [seeds, bump_seed_array].concat();

    msg!("{:?} {:?}", _pda, exchange_booth.key);

    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            exchange_booth.key,
            Rent::get()?.minimum_balance(size_of::<ExchangeBooth>()),
            size_of::<ExchangeBooth>() as u64,
            program_id,
        ),
        &[admin.clone(), exchange_booth.clone()],
        &[seeds.as_slice()],
    );

    let seeds: &[&[u8]] = &[exchange_booth.key.as_ref(), mint_a.key.as_ref()];
    let (_pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let bump_seed_array: &[&[u8]] = &[&[bump_seed]];
    let seeds = [seeds, bump_seed_array].concat();

    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            vault_a.key,
            Rent::get()?.minimum_balance(spl_token::state::Account::get_packed_len()),
            spl_token::state::Account::get_packed_len() as u64,
            &spl_token::id(),
        ),
        &[admin.clone(), vault_a.clone()],
        &[seeds.as_slice()],
    )?;
    invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            vault_a.key,
            mint_a.key,
            vault_a.key,
        )?,
        &[vault_a.clone(), mint_a.clone(), rent.clone()],
        &[seeds.as_slice()],
    )?;

    let seeds: &[&[u8]] = &[exchange_booth.key.as_ref(), mint_b.key.as_ref()];
    let (_pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let bump_seed_array: &[&[u8]] = &[&[bump_seed]];
    let seeds = [seeds, bump_seed_array].concat();

    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            vault_b.key,
            Rent::get()?.minimum_balance(spl_token::state::Account::get_packed_len()),
            spl_token::state::Account::get_packed_len() as u64,
            &spl_token::id(),
        ),
        &[admin.clone(), vault_b.clone()],
        &[seeds.as_slice()],
    )?;
    invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            vault_b.key,
            mint_b.key,
            vault_b.key,
        )?,
        &[vault_b.clone(), mint_b.clone(), rent.clone()],
        &[seeds.as_slice()],
    )?;

    ExchangeBooth {
        is_initialized: true,
        admin: *admin.key,
        vault_a: *vault_a.key,
        vault_b: *vault_b.key,
        oracle: *oracle.key,
    }.serialize(exchange_booth.try_borrow_mut_data()?.deref_mut())?;

    Ok(())
}

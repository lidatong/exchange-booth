use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::program::{invoke, invoke_signed};
use solana_program::rent::Rent;
use solana_program::stake::instruction::authorize;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, system_instruction,
};
use solana_program::bpf_loader_upgradeable::UpgradeableLoaderState::Program;

use crate::error::EchoError;
use crate::instruction::EchoInstruction;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        pub const OFFSET: usize = 9;
        let instruction = EchoInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            EchoInstruction::Echo { data } => {
                msg!("Instruction: Echo");
                let accounts = &mut accounts.iter();

                let echo = next_account_info(accounts)?;

                let mut echo_data = echo.try_borrow_mut_data()?;
                if echo_data.iter().any(|&byte| byte != 0) {
                    return Err(EchoError::Nonzero.into());
                }

                let n = echo_data.len().min(data.len());
                echo_data[..n].copy_from_slice(&data[..n]);

                Ok(())
            }
            EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed,
                buffer_size,
            } => {
                msg!("Instruction: InitializeAuthorizedEcho");
                let accounts = &mut accounts.iter();

                let authorized_buffer = next_account_info(accounts)?;
                let authority = next_account_info(accounts)?;

                let seeds: &[&[u8]] = &[
                    b"authority",
                    authority.key.as_ref(),
                    &buffer_seed.to_le_bytes(),
                ];
                let (pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
                let bump_seed_array: &[&[u8]] = &[&[bump_seed]];
                let seeds = [seeds, bump_seed_array].concat();
                invoke_signed(
                    &system_instruction::create_account(
                        authority.key,
                        &pda,
                        Rent::get()?.minimum_balance(buffer_size).max(1),
                        buffer_size as u64,
                        program_id,
                    ),
                    &[authority.clone(), authorized_buffer.clone()],
                    &[seeds.as_slice()],
                );

                let mut authorized_data = authorized_buffer.try_borrow_mut_data()?;
                authorized_data[0] = bump_seed;
                authorized_data[1..OFFSET].copy_from_slice(&buffer_seed.to_le_bytes());

                Ok(())
            }
            EchoInstruction::AuthorizedEcho { data } => {
                msg!("Instruction: AuthorizedEcho");
                let accounts = &mut accounts.iter();

                let mut authorized_buffer = next_account_info(accounts)?;
                let authority = next_account_info(accounts)?;
                if !authority.is_signer {
                    return Err(ProgramError::MissingRequiredSignature);
                }
                let mut authorized_buffer = authorized_buffer.try_borrow_mut_data()?;
                let n = authorized_buffer.len().min(data.len());
                authorized_buffer[OFFSET..].fill(0);
                authorized_buffer[OFFSET..OFFSET + n].copy_from_slice(&data[..n]);
                Ok(())
            }
            EchoInstruction::InitializeVendingMachineEcho { price, buffer_size } => {
                msg!("Instruction: InitializeVendingMachineEcho");
                let accounts = &mut accounts.iter();

                let mut vending_machine_buffer = next_account_info(accounts)?;
                let vending_machine_mint = next_account_info(accounts)?;
                let payer = next_account_info(accounts)?;

                let price_bytes = price.to_le_bytes();
                let mut seeds: Vec<&[u8]> = vec![
                    b"vending_machine",
                    vending_machine_mint.key.as_ref(),
                    &price_bytes,
                ];
                let (_pda, bump_seed) = Pubkey::find_program_address(&seeds, program_id);
                let bump_seed_array = [bump_seed];
                seeds.push(&bump_seed_array);

                invoke_signed(
                    &system_instruction::create_account(
                        payer.key,
                        vending_machine_buffer.key,
                        Rent::get()?.minimum_balance(buffer_size).max(1),
                        buffer_size as u64,
                        program_id,
                    ),
                    &[payer.clone(), vending_machine_buffer.clone()],
                    &[&seeds],
                )?;
                let mut vending_machine_buffer = vending_machine_buffer.try_borrow_mut_data()?;
                vending_machine_buffer[0] = bump_seed;
                vending_machine_buffer[1..OFFSET].copy_from_slice(&price_bytes);
                Ok(())
            }
            EchoInstruction::VendingMachineEcho { data } => {
                msg!("Instruction: VendingMachineEcho");
                let accounts = &mut accounts.iter();

                let mut vending_machine_buffer = next_account_info(accounts)?;
                let user = next_account_info(accounts)?;
                let user_token_account = next_account_info(accounts)?;
                let vending_machine_mint = next_account_info(accounts)?;

                let mut vending_machine_buffer_data =
                    vending_machine_buffer.try_borrow_mut_data()?;
                let bump_seed = vending_machine_buffer_data[0];
                let price = u64::deserialize(&mut &vending_machine_buffer_data[1..OFFSET])?;

                let seeds = &[
                    b"vending_machine",
                    vending_machine_mint.key.as_ref(),
                    &price.to_le_bytes(),
                    &[bump_seed],
                ];
                if *vending_machine_buffer.key != Pubkey::create_program_address(seeds, program_id)?
                {
                    return Err(ProgramError::InvalidAccountData);
                }
                invoke(
                    &spl_token::instruction::burn(
                        &spl_token::id(),
                        user_token_account.key,
                        vending_machine_mint.key,
                        user.key,
                        &[user.key],
                        price,
                    )?,
                    &[
                        user_token_account.clone(),
                        vending_machine_mint.clone(),
                        user.clone(),
                    ],
                );
                let n = vending_machine_buffer_data.len().min(data.len());
                vending_machine_buffer_data[OFFSET..].fill(0);
                vending_machine_buffer_data[OFFSET..OFFSET + n].copy_from_slice(&data);
                Ok(())
            }
        }
    }
}

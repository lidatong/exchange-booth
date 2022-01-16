#![cfg(feature = "test-bpf")]

use std::alloc::alloc;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use assert_matches::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Signer, transaction::Transaction};
use solana_sdk::account::ReadableAccount;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::Message;
use solana_sdk::program_error::ProgramError;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::signature::Keypair;
use solana_sdk::system_instruction;
use solana_sdk::system_program;
use solana_sdk::sysvar;
use solana_validator::test_validator::*;
use spl_token::instruction::initialize_mint;
use spl_token::state::{Account, Mint};
use echo::instruction::EchoInstruction;
use exchange_booth::instruction::ExchangeBoothInstruction;
use exchange_booth::state::{ExchangeBooth, Oracle};

#[test]
fn test_exchange_booth() -> anyhow::Result<()> {
    solana_logger::setup_with_default("solana_program_runtime=debug");
    let exchange_booth_program_id = Pubkey::new_unique();
    let echo_program_id = Pubkey::new_unique();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let user = Keypair::new();
    let admin_wallet_a = Keypair::new();
    let admin_wallet_b = Keypair::new();
    let user_wallet_a = Keypair::new();
    let user_wallet_b = Keypair::new();

    let (test_validator, admin) = TestValidatorGenesis::default()
        .add_program("exchange_booth", exchange_booth_program_id)
        .add_program("echo", echo_program_id)
        .start();
    let rpc_client = test_validator.get_rpc_client();
    // rpc_client.request_airdrop(&user.pubkey(), 1_000_000_000)?;
    // let rpc_client = RpcClient::new_with_commitment("https://api.devnet.solana.com".to_string(), CommitmentLevel::confirmed());

    let buffer_seed = 42u64;
    let (oracle, _) = Pubkey::find_program_address(
        &[
            b"authority",
            admin.pubkey().as_ref(),
            &buffer_seed.to_le_bytes(),
        ],
        &echo_program_id,
    );

    let (exchange_booth, _) = Pubkey::find_program_address(
        &[
            b"exchange_booth",
            admin.pubkey().as_ref(),
            mint_a.pubkey().as_ref(),
            mint_b.pubkey().as_ref(),
            oracle.as_ref()
        ],
        &exchange_booth_program_id,
    );

    let (vault_a, _) = Pubkey::find_program_address(
        &[
            exchange_booth.as_ref(),
            mint_a.pubkey().as_ref(),
        ],
        &exchange_booth_program_id,
    );

    let (vault_b, _) = Pubkey::find_program_address(
        &[
            exchange_booth.as_ref(),
            mint_b.pubkey().as_ref(),
        ],
        &exchange_booth_program_id,
    );

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut init_tx = Transaction::new_signed_with_payer(
        &[
            // MINTS
            system_instruction::create_account(
                &admin.pubkey(),
                &mint_a.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::get_packed_len())?,
                Mint::get_packed_len() as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint_a.pubkey(),
                &admin.pubkey(),
                None,
                0,
            )?,
            system_instruction::create_account(
                &admin.pubkey(),
                &mint_b.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::get_packed_len())?,
                spl_token::state::Mint::get_packed_len() as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint_b.pubkey(),
                &admin.pubkey(),
                None,
                0,
            )?,
            Instruction {
                program_id: exchange_booth_program_id,
                accounts: vec![
                    AccountMeta::new_readonly(admin.pubkey(), true),
                    AccountMeta::new_readonly(mint_a.pubkey(), false),
                    AccountMeta::new_readonly(mint_b.pubkey(), false),
                    AccountMeta::new(vault_a, false),
                    AccountMeta::new(vault_b, false),
                    AccountMeta::new_readonly(oracle, false),
                    AccountMeta::new(exchange_booth, false),
                    AccountMeta::new_readonly(system_program::id(), false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                    AccountMeta::new_readonly(sysvar::rent::id(), false),
                ],
                data: ExchangeBoothInstruction::InitializeExchangeBooth.try_to_vec()?,
            },
        ],
        Some(&admin.pubkey()),
        &vec![&admin, &mint_a, &mint_b],
        blockhash,
    );

    init_tx.sign(&vec![&admin, &mint_a, &mint_b], blockhash);
    rpc_client.send_and_confirm_transaction(&init_tx)?;

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut admin_wallet_tx = Transaction::new_signed_with_payer(
        &[
            // ADMIN WALLETS
            system_instruction::create_account(
                &admin.pubkey(),
                &admin_wallet_a.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Account::get_packed_len())?,
                spl_token::state::Account::get_packed_len() as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &admin_wallet_a.pubkey(),
                &mint_a.pubkey(),
                &admin.pubkey(),
            )?,
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint_a.pubkey(),
                &admin_wallet_a.pubkey(),
                &admin.pubkey(),
                &[],
                42,
            )?,
            system_instruction::create_account(
                &admin.pubkey(),
                &admin_wallet_b.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Account::get_packed_len())?,
                spl_token::state::Account::get_packed_len() as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &admin_wallet_b.pubkey(),
                &mint_b.pubkey(),
                &admin.pubkey(),
            )?,
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint_b.pubkey(),
                &admin_wallet_b.pubkey(),
                &admin.pubkey(),
                &[],
                42,
            )?,
        ],
        Some(&admin.pubkey()),
        &vec![&admin, &admin_wallet_a, &admin_wallet_b],
        blockhash,
    );
    admin_wallet_tx.sign(&vec![&admin, &admin_wallet_a, &admin_wallet_b], blockhash);
    rpc_client.send_and_confirm_transaction(&admin_wallet_tx)?;

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut user_wallet_tx = Transaction::new_signed_with_payer(
        &[
            // USER WALLETS
            system_instruction::create_account(
                &admin.pubkey(),
                &user_wallet_a.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Account::get_packed_len())?,
                spl_token::state::Account::get_packed_len() as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &user_wallet_a.pubkey(),
                &mint_a.pubkey(),
                &user.pubkey(),
            )?,
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint_a.pubkey(),
                &user_wallet_a.pubkey(),
                &admin.pubkey(),
                &[],
                42,
            )?,
            system_instruction::create_account(
                &admin.pubkey(),
                &user_wallet_b.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Account::get_packed_len())?,
                spl_token::state::Account::get_packed_len() as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &user_wallet_b.pubkey(),
                &mint_b.pubkey(),
                &user.pubkey(),
            )?,
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint_b.pubkey(),
                &user_wallet_b.pubkey(),
                &admin.pubkey(),
                &[],
                42,
            )?,
        ],
        Some(&admin.pubkey()),
        &vec![&user_wallet_a, &user_wallet_b, &admin],
        blockhash,
    );
    user_wallet_tx.sign(&vec![&user_wallet_a, &user_wallet_b, &admin], blockhash);
    rpc_client.send_and_confirm_transaction(&user_wallet_tx)?;

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut deposit_tx = Transaction::new_signed_with_payer(
        &[
            // DEPOSIT
            Instruction {
                program_id: exchange_booth_program_id,
                accounts: vec![
                    AccountMeta::new(admin_wallet_a.pubkey(), false),
                    AccountMeta::new(vault_a, false),
                    AccountMeta::new_readonly(admin.pubkey(), true),
                    AccountMeta::new_readonly(exchange_booth, false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: ExchangeBoothInstruction::Deposit { amount: 42 }.try_to_vec()?,
            },
        ],
        Some(&admin.pubkey()),
        &vec![&admin],
        blockhash,
    );
    deposit_tx.sign(&vec![&admin], blockhash);
    rpc_client.send_and_confirm_transaction(&deposit_tx)?;


    println!("--- Initialize Exchange Booth ---");
    println!("admin: {:?}\nmint_a: {:?}\nmint_b: {:?}\noracle: {:?}\nexchange_booth: {:?}", admin.pubkey(), mint_a.pubkey(), mint_b.pubkey(), oracle, exchange_booth);
    let exchange_booth_data = ExchangeBooth::try_from_slice(&rpc_client.get_account(&exchange_booth)?.data)?;
    println!("{:?}", exchange_booth_data);
    println!();
    let vault_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_a)?.data)?;
    let vault_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_b)?.data)?;
    let wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_a.pubkey())?.data)?;
    let wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_b.pubkey())?.data)?;
    let user_wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_a.pubkey())?.data)?;
    let user_wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_b.pubkey())?.data)?;
    println!("\u{001b}[36m vault_a \u{001b}[0m {:?}", vault_a_account);
    println!("\u{001b}[36m vault_b \u{001b}[0m {:?}", vault_b_account);
    println!("\u{001b}[36m admin_wallet_a \u{001b}[0m {:?}", wallet_a_account);
    println!("\u{001b}[36m admin_wallet_b \u{001b}[0m {:?}", wallet_b_account);
    println!("\u{001b}[36m user_wallet_a \u{001b}[0m {:?}", user_wallet_a_account);
    println!("\u{001b}[36m user_wallet_b \u{001b}[0m {:?}", user_wallet_b_account);
    println!();

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut deposit_tx = Transaction::new_signed_with_payer(
        &[
            // DEPOSIT
            Instruction {
                program_id: exchange_booth_program_id,
                accounts: vec![
                    AccountMeta::new(admin_wallet_b.pubkey(), false),
                    AccountMeta::new(vault_b, false),
                    AccountMeta::new_readonly(admin.pubkey(), true),
                    AccountMeta::new_readonly(exchange_booth, false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: ExchangeBoothInstruction::Deposit { amount: 42 }.try_to_vec()?,
            },
        ],
        Some(&admin.pubkey()),
        &vec![&admin],
        blockhash,
    );
    deposit_tx.sign(&vec![&admin], blockhash);
    rpc_client.send_and_confirm_transaction(&deposit_tx)?;

    println!("--- Deposit ---");
    let vault_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_a)?.data)?;
    let vault_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_b)?.data)?;
    let wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_a.pubkey())?.data)?;
    let wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_b.pubkey())?.data)?;
    let user_wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_a.pubkey())?.data)?;
    let user_wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_b.pubkey())?.data)?;
    println!("\u{001b}[36m vault_a \u{001b}[0m {:?}", vault_a_account);
    println!("\u{001b}[36m vault_b \u{001b}[0m {:?}", vault_b_account);
    println!("\u{001b}[36m admin_wallet_a \u{001b}[0m {:?}", wallet_a_account);
    println!("\u{001b}[36m admin_wallet_b \u{001b}[0m {:?}", wallet_b_account);
    println!("\u{001b}[36m user_wallet_a \u{001b}[0m {:?}", user_wallet_a_account);
    println!("\u{001b}[36m user_wallet_b \u{001b}[0m {:?}", user_wallet_b_account);
    println!();

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut withdraw_tx = Transaction::new_signed_with_payer(
        &[
            // WITHDRAW
            Instruction {
                program_id: exchange_booth_program_id,
                accounts: vec![
                    AccountMeta::new(vault_a, false),
                    AccountMeta::new(admin_wallet_a.pubkey(), false),
                    AccountMeta::new_readonly(mint_a.pubkey(), false),
                    AccountMeta::new_readonly(admin.pubkey(), true),
                    AccountMeta::new_readonly(exchange_booth, false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: ExchangeBoothInstruction::Withdraw { amount: 21 }.try_to_vec()?,
            },
        ],
        Some(&admin.pubkey()),
        &vec![&admin],
        blockhash,
    );
    withdraw_tx.sign(&vec![&admin], blockhash);
    rpc_client.send_and_confirm_transaction(&withdraw_tx)?;

    println!("--- Withdraw ---");
    let vault_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_a)?.data)?;
    let vault_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_b)?.data)?;
    let wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_a.pubkey())?.data)?;
    let wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_b.pubkey())?.data)?;
    let user_wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_a.pubkey())?.data)?;
    let user_wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_b.pubkey())?.data)?;
    println!("\u{001b}[36m vault_a \u{001b}[0m {:?}", vault_a_account);
    println!("\u{001b}[36m vault_b \u{001b}[0m {:?}", vault_b_account);
    println!("\u{001b}[36m admin_wallet_a \u{001b}[0m {:?}", wallet_a_account);
    println!("\u{001b}[36m admin_wallet_b \u{001b}[0m {:?}", wallet_b_account);
    println!("\u{001b}[36m user_wallet_a \u{001b}[0m {:?}", user_wallet_a_account);
    println!("\u{001b}[36m user_wallet_b \u{001b}[0m {:?}", user_wallet_b_account);
    println!();

    let blockhash = rpc_client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id: echo_program_id,
            accounts: vec![
                AccountMeta::new(oracle, false),
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed,
                buffer_size: 17,
            }.try_to_vec()?,
        }],
        Some(&admin.pubkey()),
        &vec![&admin],
        blockhash,
    );
    transaction.sign(&[&admin], blockhash);
    rpc_client.send_and_confirm_transaction(&transaction)?;
    let account = rpc_client.get_account(&oracle)?;

    let blockhash = rpc_client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id: echo_program_id,
            accounts: vec![
                AccountMeta::new(oracle, false),
                AccountMeta::new(admin.pubkey(), true),
            ],
            data: EchoInstruction::AuthorizedEcho {
                data: Oracle { exchange_rate: 2 }.try_to_vec()?
            }.try_to_vec()?,
        }],
        Some(&admin.pubkey()),
        &vec![&admin],
        blockhash,
    );
    transaction.sign(&[&admin], blockhash);
    rpc_client.send_and_confirm_transaction(&transaction)?;
    let account = rpc_client.get_account(&oracle)?;
    println!("--- Oracle ---");
    println!("{:?}", Oracle { exchange_rate: 2 });
    println!();

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut exchange_tx = Transaction::new_signed_with_payer(
        &[
            // EXCHANGE
            Instruction {
                program_id: exchange_booth_program_id,
                accounts: vec![
                    AccountMeta::new(user_wallet_a.pubkey(), false),
                    AccountMeta::new(vault_a, false),
                    AccountMeta::new(vault_b, false),
                    AccountMeta::new(user_wallet_b.pubkey(), false),
                    AccountMeta::new_readonly(mint_b.pubkey(), false),
                    AccountMeta::new_readonly(user.pubkey(), true),
                    AccountMeta::new_readonly(oracle, false),
                    AccountMeta::new_readonly(exchange_booth, false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: ExchangeBoothInstruction::Exchange { amount: 21 }.try_to_vec()?,
            },
        ],
        Some(&admin.pubkey()),
        &vec![&admin, &user],
        blockhash,
    );
    exchange_tx.sign(&vec![&admin, &user], blockhash);
    rpc_client.send_and_confirm_transaction(&exchange_tx)?;

    println!("--- Exchange ---");
    let vault_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_a)?.data)?;
    let vault_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&vault_b)?.data)?;
    let wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_a.pubkey())?.data)?;
    let wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&admin_wallet_b.pubkey())?.data)?;
    let user_wallet_a_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_a.pubkey())?.data)?;
    let user_wallet_b_account = spl_token::state::Account::unpack(&rpc_client.get_account(&user_wallet_b.pubkey())?.data)?;
    println!("\u{001b}[36m vault_a \u{001b}[0m {:?}", vault_a_account);
    println!("\u{001b}[36m vault_b \u{001b}[0m {:?}", vault_b_account);
    println!("\u{001b}[36m admin_wallet_a \u{001b}[0m {:?}", wallet_a_account);
    println!("\u{001b}[36m admin_wallet_b \u{001b}[0m {:?}", wallet_b_account);
    println!("\u{001b}[36m user_wallet_a \u{001b}[0m {:?}", user_wallet_a_account);
    println!("\u{001b}[36m user_wallet_b \u{001b}[0m {:?}", user_wallet_b_account);

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut close_tx = Transaction::new_signed_with_payer(
        &[
            // CLOSE
            Instruction {
                program_id: exchange_booth_program_id,
                accounts: vec![
                    AccountMeta::new(admin.pubkey(), true),
                    AccountMeta::new(exchange_booth, false),
                ],
                data: ExchangeBoothInstruction::CloseExchangeBooth.try_to_vec()?,
            },
        ],
        Some(&admin.pubkey()),
        &vec![&admin],
        blockhash,
    );
    close_tx.sign(&vec![&admin], blockhash);
    rpc_client.send_and_confirm_transaction(&close_tx)?;
    Ok(())
}


use std::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Oracle {
    pub exchange_rate: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ExchangeBooth {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub oracle: Pubkey,
}
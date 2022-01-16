use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

// TODO numeric overflow / rounding

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ExchangeBoothInstruction {
    // named arguments?
    InitializeExchangeBooth,
    Deposit {
        amount: u64
    },
    Withdraw {
        amount: u64
    },
    Exchange {
        amount: u64
    },
    CloseExchangeBooth,
}

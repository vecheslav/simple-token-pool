//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

/// Instructions supported by the program
#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum PoolInstruction {
    /// Initializes a new program
    Initialize,

    /// Swap tokens
    Swap {
        /// Amount of token IN
        amount_in: u64,
    },
}

/// Creates 'Initialize' instruction.
pub fn initialize(
    program_id: &Pubkey,
    pool: &Pubkey,
    authority: &Pubkey,
    token_program_id: &Pubkey,
    bank_mint: &Pubkey,
    pool_mint: &Pubkey,
    bank: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*pool, true),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(*bank_mint, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new(*bank, false),
    ];

    Ok(Instruction::new_with_borsh(
        *program_id,
        &PoolInstruction::Initialize,
        accounts,
    ))
}

/// Creates 'Swap' instruction.
pub fn swap(
    program_id: &Pubkey,
    pool: &Pubkey,
    pool_authority: &Pubkey,
    token_program_id: &Pubkey,
    pool_mint: &Pubkey,
    bank_mint: &Pubkey,
    bank: &Pubkey,
    sender: &Pubkey,
    amount_in: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*pool, false),
        AccountMeta::new_readonly(*pool_authority, false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*bank_mint, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*sender, true),
    ];

    Ok(Instruction::new_with_borsh(
        *program_id,
        &PoolInstruction::Swap { amount_in },
        accounts,
    ))
}

//! Program entrypoint

#![cfg(not(feature = "no-entrypoint"))]

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) =
        crate::processor::process_instruction(program_id, accounts, instruction_data)
    {
        msg!("Error: {:?}", error);
        return Err(error);
    }
    Ok(())
}

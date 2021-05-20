//! Program state processor

use crate::{find_authority_bump_seed, instruction::PoolInstruction, state::PoolData};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token as token;

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = PoolInstruction::try_from_slice(input)?;
    let account_info_iter = &mut accounts.iter();

    match instruction {
        PoolInstruction::Initialize => {
            msg!("PoolInstruction::Initialize");

            let pool_info = next_account_info(account_info_iter)?;
            let authority_info = next_account_info(account_info_iter)?;
            let bank_mint_info = next_account_info(account_info_iter)?;
            let pool_mint_info = next_account_info(account_info_iter)?;
            let bank_info = next_account_info(account_info_iter)?;
            let rent_info = next_account_info(account_info_iter)?;
            // let token_program_info = next_account_info(account_info_iter)?;

            let rent = &Rent::from_account_info(rent_info)?;

            if pool_info.owner != program_id {
                return Err(ProgramError::IncorrectProgramId);
            }

            let mut pool_data = PoolData::try_from_slice(&pool_info.data.borrow())?;
            if pool_data.is_initialized() {
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            let bank_mint = token::state::Mint::unpack_from_slice(&bank_mint_info.data.borrow())?;
            if !bank_mint.is_initialized() {
                return Err(ProgramError::UninitializedAccount);
            }

            // Check rent
            for account in &[pool_info, bank_info, pool_mint_info] {
                if !rent.is_exempt(account.lamports(), account.data_len()) {
                    return Err(ProgramError::AccountNotRentExempt);
                }
            }

            // Calculate authority address
            let (authority, bump_seed) = find_authority_bump_seed(program_id, &pool_info.key);
            if authority != *authority_info.key {
                return Err(ProgramError::InvalidArgument);
            }

            // Initialize account for spl token
            spl_initialize_account(
                bank_info.clone(),
                bank_mint_info.clone(),
                authority_info.clone(),
                rent_info.clone(),
            )?;

            // Initialize mint (token) for pool
            spl_initialize_mint(
                pool_mint_info.clone(),
                authority_info.clone(),
                rent_info.clone(),
                bank_mint.decimals,
            )?;

            pool_data.version = PoolData::CURRENT_VERSION;
            pool_data.authority = *authority_info.key;
            // pool_data.token_program_id = *token_program_info.key;
            pool_data.bank_mint = *bank_mint_info.key;
            pool_data.pool_mint = *pool_mint_info.key;
            pool_data.bank = *bank_info.key;
            pool_data.bump_seed = bump_seed;

            pool_data.serialize(&mut *pool_info.data.borrow_mut())?;
        }
        PoolInstruction::Swap { amount_in } => {
            msg!("PoolInstruction::Swap");

            let pool_info = next_account_info(account_info_iter)?;
            let pool_authority_info = next_account_info(account_info_iter)?;
            let user_transfer_authority_info = next_account_info(account_info_iter)?;
            let pool_mint_info = next_account_info(account_info_iter)?;
            let bank_info = next_account_info(account_info_iter)?;
            let sender_info = next_account_info(account_info_iter)?;
            let recipient_info = next_account_info(account_info_iter)?;
            // let token_program_info = next_account_info(account_info_iter)?;

            let pool_data = PoolData::try_from_slice(&pool_info.data.borrow())?;

            if !pool_data.is_initialized() {
                return Err(ProgramError::UninitializedAccount);
            }

            // Check autority
            pool_data.check_authority(pool_authority_info.key, program_id, pool_info.key)?;

            if pool_data.bank != *bank_info.key {
                return Err(ProgramError::InvalidArgument);
            }

            // Transfer savings tokens from user
            spl_token_transfer(
                pool_info.key,
                sender_info.clone(),
                bank_info.clone(),
                user_transfer_authority_info.clone(),
                pool_data.bump_seed,
                amount_in,
            )?;

            // Mint pool tokens to user
            spl_token_mint_to(
                pool_info.key,
                pool_mint_info.clone(),
                recipient_info.clone(),
                pool_authority_info.clone(),
                pool_data.bump_seed,
                amount_in * (PoolData::MINT_MULTIPLIER as u64),
            )?;
        }
    }

    Ok(())
}

/// Create a mint instruction.
pub fn spl_initialize_mint<'a>(
    mint: AccountInfo<'a>,
    mint_authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    decimals: u8,
) -> Result<(), ProgramError> {
    let ix = token::instruction::initialize_mint(
        &token::id(),
        mint.key,
        mint_authority.key,
        None,
        decimals,
    )?;

    invoke(&ix, &[mint, rent])
}

/// Create an accont instruction.
pub fn spl_initialize_account<'a>(
    account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
) -> Result<(), ProgramError> {
    let ix =
        token::instruction::initialize_account(&token::id(), account.key, mint.key, authority.key)?;

    invoke(&ix, &[account, mint, authority, rent])
}

/// Issue a transfer instruction.
pub fn spl_token_transfer<'a>(
    pool: &Pubkey,
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump_seed: u8,
    amount: u64,
) -> Result<(), ProgramError> {
    let authority_signature_seeds = [&pool.to_bytes()[..32], &[bump_seed]];
    let signers = &[&authority_signature_seeds[..]];

    let ix = token::instruction::transfer(
        &token::id(),
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[source, destination, authority], signers)
}

/// Issue a mint instruction.
pub fn spl_token_mint_to<'a>(
    pool: &Pubkey,
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump_seed: u8,
    amount: u64,
) -> Result<(), ProgramError> {
    let authority_signature_seeds = [&pool.to_bytes()[..32], &[bump_seed]];
    let signers = &[&authority_signature_seeds[..]];

    let ix = token::instruction::mint_to(
        &token::id(),
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[mint, destination, authority], signers)
}

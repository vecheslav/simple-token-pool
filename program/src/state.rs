//! State transition types

use solana_program::{msg, program_error::ProgramError};

use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{program_pack::IsInitialized, pubkey::Pubkey},
};

/// Program states
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PoolData {
    /// Struct version, allows for upgrades to the program
    pub version: u8,

    /// The account allowed to update the data
    pub authority: Pubkey,

    /// Bump seed
    pub bump_seed: u8,

    /// Mint for the tokens sent to the pool
    pub bank_mint: Pubkey,

    /// Mint for sending tokens to user
    pub pool_mint: Pubkey,

    /// Account for tokens from user
    pub bank: Pubkey,
}

impl PoolData {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u8 = 1;

    /// Mint multiplier
    pub const MINT_MULTIPLIER: u8 = 1;

    /// Checks that the withdraw or deposit authority is valid
    pub(crate) fn check_authority(
        &self,
        authority_address: &Pubkey,
        program_id: &Pubkey,
        pool: &Pubkey,
    ) -> Result<(), ProgramError> {
        let expected_address = Pubkey::create_program_address(
            &[&pool.to_bytes()[..32], &[self.bump_seed]],
            program_id,
        )?;

        if *authority_address == expected_address {
            Ok(())
        } else {
            msg!(
                "Incorrect authority provided, expected {}, received {}",
                expected_address,
                authority_address
            );
            Err(ProgramError::InvalidArgument)
        }
    }
}

impl IsInitialized for PoolData {
    /// Is initialized
    fn is_initialized(&self) -> bool {
        self.version == Self::CURRENT_VERSION
    }
}

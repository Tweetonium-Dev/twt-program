use core::mem::transmute;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    states::ProjectV1,
    utils::{AccountCheck, InitPdaAccounts, InitPdaArgs, Pda, UninitializedAccount},
};

/// Tracks whether a specific wallet has already minted an NFT in this collection.
///
/// Each record represents a single user’s mint eligibility.
/// Used to enforce per-wallet mint limits or prevent double-minting.
///
/// PDA seed: `[program_id, payer, "minted_user"]`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserMintedV1 {
    /// The wallet address of the user.
    /// Must match the `payer` in the `mint_and_vault_v1` instruction.
    pub owner: Pubkey,

    /// The total number of NFTs minted by this wallet.
    ///
    /// - Starts at `1` when the record is first initialized.
    /// - Incremented atomically on each successful user mint.
    /// - Must never exceed `Config::max_user_minted`.
    ///
    /// Used to enforce per-user mint caps and prevent over-minting.
    pub minted_count: u64,
}

impl UserMintedV1 {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &[u8; 14] = b"user_minted_v1";
}

impl UserMintedV1 {
    #[inline(always)]
    pub fn init<'a, 'info>(
        accounts: InitUserMintedAccounts<'a, 'info>,
        args: InitUserMintedArgs<'a>,
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
    ) -> Result<(), ProgramError> {
        Pda::new(pda_accounts, pda_args)?.init()?;

        let mut bytes = accounts.pda.try_borrow_mut_data()?;

        let minted_user = Self::load_mut(&mut bytes)?;
        minted_user.owner = *args.owner;
        minted_user.minted_count = 0;

        Ok(())
    }

    #[inline(always)]
    pub fn init_if_needed<'a, 'info>(
        accounts: InitUserMintedAccounts<'a, 'info>,
        args: InitUserMintedArgs<'a>,
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
    ) -> Result<(), ProgramError> {
        if UninitializedAccount::check(pda_accounts.pda).is_ok() {
            Self::init(accounts, args, pda_accounts, pda_args)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() < Self::LEN {
            msg!("Load mutable UserMinted: invalid account data length");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    #[inline(always)]
    pub fn has_reached_limit(&self, config: &ProjectV1) -> bool {
        if config.max_mint_per_user == 0 {
            return false;
        }
        self.minted_count >= config.max_mint_per_user
    }

    #[inline(always)]
    pub fn has_reached_vip_limit(&self, config: &ProjectV1) -> bool {
        if config.max_mint_per_vip_user == 0 {
            return false;
        }
        self.minted_count >= config.max_mint_per_vip_user
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        self.minted_count = self.minted_count.saturating_add(1);
    }
}

pub struct InitUserMintedAccounts<'a, 'info> {
    pub pda: &'a AccountInfo<'info>,
}

pub struct InitUserMintedArgs<'a> {
    pub owner: &'a Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test Helpers ---

    fn zero_user_minted() -> Vec<u8> {
        vec![0u8; UserMintedV1::LEN]
    }

    fn zero_config() -> Vec<u8> {
        vec![0u8; ProjectV1::LEN]
    }

    // --- Test Cases ---

    #[test]
    fn test_user_minted_load_mut_and_increment() {
        let mut data = zero_user_minted();
        let minted = UserMintedV1::load_mut(&mut data).unwrap();

        // Should initialize to default values
        minted.owner = Pubkey::new_unique();
        assert_eq!(minted.minted_count, 0);

        minted.increment();
        assert_eq!(minted.minted_count, 1);

        // Saturating increment test
        minted.minted_count = u64::MAX;
        minted.increment();
        assert_eq!(minted.minted_count, u64::MAX);
    }

    #[test]
    fn test_user_minted_has_reached_limit() {
        let mut buf = zero_config();
        let config = ProjectV1::load_mut(&mut buf).expect("load_mut should succeed");
        config.max_mint_per_user = 3;
        config.max_mint_per_vip_user = 10;

        let mut user = UserMintedV1 {
            owner: Pubkey::new_unique(),
            minted_count: 2,
        };

        assert!(!user.has_reached_limit(config));
        assert!(!user.has_reached_vip_limit(config));

        user.minted_count = 3;
        assert!(user.has_reached_limit(config));
        assert!(!user.has_reached_vip_limit(config));

        user.minted_count = 10;
        assert!(user.has_reached_vip_limit(config));
    }

    #[test]
    fn test_user_minted_invalid_data_length() {
        let mut short_data = vec![0u8; UserMintedV1::LEN - 1];
        let err = UserMintedV1::load_mut(&mut short_data);
        assert!(err.is_err());
    }
}

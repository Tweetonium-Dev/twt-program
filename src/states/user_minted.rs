use core::mem::transmute;
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    states::Config,
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
pub struct UserMinted {
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

impl UserMinted {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &[u8; 11] = b"user_minted";
}

impl UserMinted {
    #[inline(always)]
    pub fn init<'a, 'info>(
        bytes: &mut [u8],
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
        owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        Pda::new(pda_accounts, pda_args)?.init()?;

        let minted_user = Self::load_mut(bytes)?;
        minted_user.owner = *owner;
        minted_user.minted_count = 1;

        Ok(())
    }

    #[inline(always)]
    pub fn init_if_needed<'a, 'info>(
        bytes: &mut [u8],
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
        owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        if UninitializedAccount::check(pda_accounts.pda).is_ok() {
            Self::init(bytes, pda_accounts, pda_args, owner)?;
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
    pub fn has_reached_limit(&self, config: &Config) -> bool {
        if config.max_mint_per_user == 0 {
            return false;
        }
        self.minted_count >= config.max_mint_per_user
    }

    #[inline(always)]
    pub fn has_reached_vip_limit(&self, config: &Config) -> bool {
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

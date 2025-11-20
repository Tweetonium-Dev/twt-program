use core::mem::transmute;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{AccountCheck, InitPdaAccounts, InitPdaArgs, Pda, UninitializedAccount};

/// Represents the escrow state for a minted NFT and its associated SPL tokens.
///
/// A vault is created for every minted NFT, holding the user's escrowed ZDLT tokens
/// until the vesting period ends. Once unlocked, the user can burn their NFT
/// and reclaim the escrowed tokens.
///
/// PDA seed: `[program_id, config_pda, payer, "vault"]`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VaultV1 {
    /// The MPL Core NFT asset (mint) that corresponds to this vault.
    /// Used to verify the correct NFT is being burned during `burn_and_refund_v1`.
    pub nft: Pubkey,

    /// The total amount of ZDLT tokens escrowed for this NFT (raw units).
    ///
    /// Matches the `config.escrow_amount` at mint time.
    /// These tokens are locked until vesting unlocks, then refunded to `owner`
    /// when the NFT is burned.
    pub amount: u64,

    /// Flag indicating whether the escrowed tokens have been released.
    ///
    /// - `0` = tokens still locked (default)
    /// - `1` = tokens have been withdrawn
    ///
    /// Set to `1` atomically in `burn_and_refund_v1` after the refund transfer.
    pub is_unlocked: u8,

    /// The bump seed used when deriving the vault PDA (`["vault", config_pda]`).
    ///
    /// Stored for replay protection and deterministic PDA re-derivation.
    pub bump: [u8; 1],
}

impl VaultV1 {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &[u8; 8] = b"vault_v1";
}

impl VaultV1 {
    #[inline(always)]
    pub fn init<'a, 'info>(
        accounts: InitVaultAccounts,
        args: InitVaultArgs,
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
    ) -> ProgramResult {
        let bump = Pda::new(pda_accounts, pda_args)?.init()?;

        let mut bytes = accounts.pda.try_borrow_mut_data()?;

        let vault = Self::load_mut(&mut bytes)?;
        vault.nft = args.nft;
        vault.amount = args.amount;
        vault.is_unlocked = if args.is_unlocked { 1 } else { 0 };
        vault.bump = [bump];

        Ok(())
    }

    #[inline(always)]
    pub fn init_if_needed<'a, 'info>(
        accounts: InitVaultAccounts,
        args: InitVaultArgs,
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
    ) -> ProgramResult {
        if UninitializedAccount::check(pda_accounts.pda).is_ok() {
            Self::init(accounts, args, pda_accounts, pda_args)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            msg!("Load mut vault with wrong bytes length");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            msg!("Load vault with wrong bytes length");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; Self::LEN];

        unsafe {
            std::ptr::copy_nonoverlapping(
                self as *const Self as *const u8,
                bytes.as_mut_ptr(),
                Self::LEN,
            );
        }

        bytes
    }

    #[inline(always)]
    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked == 1
    }
}

pub struct InitVaultAccounts<'a, 'info> {
    pub pda: &'a AccountInfo<'info>,
}

pub struct InitVaultArgs {
    pub nft: Pubkey,
    pub amount: u64,
    pub is_unlocked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test Helpers ---

    fn zero_vault() -> Vec<u8> {
        vec![0u8; VaultV1::LEN]
    }

    // --- Test Cases ---

    #[test]
    fn test_vault_load_and_load_mut() {
        let mut data = zero_vault();
        let vault_mut = VaultV1::load_mut(&mut data).unwrap();
        vault_mut.nft = Pubkey::new_unique();
        vault_mut.amount = 42;
        vault_mut.is_unlocked = 1;
        vault_mut.bump = [123];

        let vault_ref = VaultV1::load(&data).unwrap();

        assert_eq!(vault_ref.amount, 42);
        assert!(vault_ref.is_unlocked());
        assert_eq!(vault_ref.bump, [123]);
    }

    #[test]
    fn test_vault_load_invalid_length() {
        let mut bad = vec![0u8; VaultV1::LEN - 1];
        assert!(VaultV1::load(&bad).is_err());
        assert!(VaultV1::load_mut(&mut bad).is_err());
    }

    #[test]
    fn test_vault_is_unlocked() {
        let locked = VaultV1 {
            nft: Pubkey::new_unique(),
            amount: 10,
            is_unlocked: 0,
            bump: [0],
        };
        let unlocked = VaultV1 {
            is_unlocked: 1,
            ..locked
        };
        assert!(!locked.is_unlocked());
        assert!(unlocked.is_unlocked());
    }
}

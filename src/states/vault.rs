use core::mem::transmute;
use solana_program::{entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};

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
pub struct Vault {
    /// The wallet that owns this vault and its escrowed tokens.
    /// Must match the `payer` in the `mint_and_vault_v1` instruction.
    pub owner: Pubkey,

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

impl Vault {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &[u8; 5] = b"vault";
}

impl Vault {
    #[inline(always)]
    pub fn init<'a, 'info>(
        bytes: &mut [u8],
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
        args: InitVaultArgs,
    ) -> ProgramResult {
        let bump = Pda::new(pda_accounts, pda_args)?.init()?;

        let vault = Self::load_mut(bytes)?;
        vault.owner = args.owner;
        vault.nft = args.nft;
        vault.amount = args.amount;
        vault.is_unlocked = if args.is_unlocked { 1 } else { 0 };
        vault.bump = [bump];

        Ok(())
    }

    #[inline(always)]
    pub fn init_if_needed<'a, 'info>(
        bytes: &mut [u8],
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
        args: InitVaultArgs,
    ) -> ProgramResult {
        if UninitializedAccount::check(pda_accounts.pda).is_ok() {
            Self::init(bytes, pda_accounts, pda_args, args)?;
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
    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked == 1
    }
}

pub struct InitVaultArgs {
    pub owner: Pubkey,
    pub nft: Pubkey,
    pub amount: u64,
    pub is_unlocked: bool,
}

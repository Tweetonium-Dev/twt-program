use core::mem::transmute;
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{MAX_BASIS_POINTS, MAX_ROYALTY_RECIPIENTS},
    utils::{AccountCheck, InitPdaAccounts, InitPdaArgs, Pda, UninitializedAccount},
};

/// Global configuration account that defines minting, payment, and vesting rules
/// for a single collection or minting campaign.
///
/// This account is initialized once via `init_config_v1` and governs:
/// - The payment token and price model (SPL mint, escrow, DAO shares)
/// - The maximum mint supply and whitelist (WL) phase logic
/// - Vesting parameters for escrowed tokens (time-based or off-chain unlock)
/// - Royalty and DAO revenue splits
///
/// Each `Vault` and `MintedUser` record derives from this `Config` using its PDA.
///
/// PDA seed: `[program_id, "config", hashed_nft_symbol, token_mint]`
#[repr(C)]
#[derive(Debug, Clone, Copy, ShankAccount)]
pub struct TraitItem {
    /// The authority that controls configuration updates and protocol-level actions.
    ///
    /// - Must match the signer in `update_trait_v1`.
    /// - Used to validate authority-only instructions.
    pub authority: Pubkey,

    /// The absolute cap on NFTs that can ever be created (Mac number of admin_minted + user_minted).
    ///
    /// - This is the global maximum supply for the collection.
    /// - `max_supply - released` is implicitly reserved for DAO/admin mints (not available to public users).
    /// - `admin_minted + user_minted` must never exceed this value.
    pub max_supply: u64,

    /// Current number of NFTs minted (user mints).
    ///
    /// - Incremented atomically on each successful user mint.
    /// - Enforced to never exceed `released`.
    pub user_minted: u64,

    /// The SOL protocol fee (in lamports) charged on each mint.
    ///
    /// - Transferred to the protocol’s treasury wallet.
    /// - Example: `500_000` lamports = 0.0005 SOL.
    pub mint_fee_lamports: u64,
}

impl TraitItem {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &[u8; 10] = b"trait_item";
}

impl TraitItem {
    #[inline(always)]
    pub fn init<'a, 'info>(
        accounts: InitTraitItemAccounts<'a, 'info>,
        args: InitTraitItemArgs,
        pda_accounts: InitPdaAccounts<'a, 'info>,
        pda_args: InitPdaArgs<'a>,
    ) -> ProgramResult {
        Pda::new(pda_accounts, pda_args)?.init()?;

        let mut bytes = accounts.pda.try_borrow_mut_data()?;

        let config = Self::load_mut(&mut bytes)?;
        config.authority = args.authority;
        config.max_supply = args.max_supply;
        config.user_minted = args.user_minted;
        config.mint_fee_lamports = args.mint_fee_lamports;

        Ok(())
    }

    #[inline(always)]
    pub fn init_if_needed<'a, 'info>(
        accounts: InitTraitItemAccounts<'a, 'info>,
        args: InitTraitItemArgs,
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
        if bytes.len() < Self::LEN {
            msg!("Load mut trait item account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    #[inline(always)]
    pub fn is_free_mint_fee(&self) -> bool {
        self.mint_fee_lamports == 0
    }

    #[inline(always)]
    pub fn stock_available(&self) -> bool {
        self.user_minted <= self.max_supply
    }

    #[inline(always)]
    pub fn increment_user_minted(&mut self) -> ProgramResult {
        self.user_minted = self
            .user_minted
            .checked_add(1)
            .inspect(|_| msg!("Unable to increment config.user_minted"))
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(())
    }

    #[inline(always)]
    pub fn check_trait_royalties(
        num_royalty_recipients: u8,
        royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
        royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    ) -> ProgramResult {
        let recipients = num_royalty_recipients as usize;

        if recipients == 0 {
            return Ok(());
        }

        if recipients > MAX_ROYALTY_RECIPIENTS {
            msg!("Too many royalty wallets, max: {}", MAX_ROYALTY_RECIPIENTS);
            return Err(ProgramError::InvalidInstructionData);
        }

        let input_recipients_count = royalty_recipients
            .iter()
            .filter(|pk| **pk != Pubkey::default())
            .count();

        let input_shares_count = royalty_shares_bps.iter().filter(|s| **s != 0).count();

        if recipients != input_recipients_count || recipients != input_shares_count {
            msg!(
                "Royalty mismatch: declared {} recipients, but found {} valid wallets and {} non-zero share entries",
                recipients,
                input_recipients_count,
                input_shares_count,
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        let total_bps: u16 = royalty_shares_bps
            .iter()
            .try_fold(0u16, |acc, &price| {
                acc.checked_add(price)
                    .ok_or(ProgramError::InvalidInstructionData)
            })
            .inspect_err(|_| msg!("Overflow while summing total basis points"))?;

        if total_bps > MAX_BASIS_POINTS {
            msg!("Total royalty basis points exceeds 100% (10_000)");
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn update(&mut self, args: UpdateTraitItemArgs) {
        self.max_supply = args.max_supply;
        self.mint_fee_lamports = args.mint_fee_lamports;
    }
}

pub struct InitTraitItemAccounts<'a, 'info> {
    pub pda: &'a AccountInfo<'info>,
}

pub struct InitTraitItemArgs {
    pub authority: Pubkey,
    pub max_supply: u64,
    pub user_minted: u64,
    pub mint_fee_lamports: u64,
}

pub struct UpdateTraitItemArgs {
    pub max_supply: u64,
    pub mint_fee_lamports: u64,
}

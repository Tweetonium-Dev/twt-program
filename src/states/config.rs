use core::mem::transmute;
use solana_program::{entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    states::{VestingMode, MAX_BASIS_POINTS, MAX_REVENUE_WALLETS, MAX_ROYALTY_RECIPIENTS},
    utils::{AccountCheck, InitPdaArgs, Pda, UninitializedAccount},
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
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The authority that controls configuration updates and protocol-level actions.
    ///
    /// - Must match the signer in `update_config_v1`.
    /// - Used to validate admin-only instructions (e.g. force unlocks).
    pub admin: Pubkey,

    /// The SPL token mint (e.g. ZDLT) used as the payment and escrow token.
    ///
    /// - Must match `token_mint` in `mint_and_vault_v1`.
    /// - All payments, vault escrows, and refunds use this mint.
    pub mint: Pubkey,

    /// The number of decimal places used by the payment mint (e.g. `6` for ZDLT).
    ///
    /// - Used to normalize on-chain arithmetic and enforce exact token amounts.
    pub mint_decimals: u8,

    /// The absolute cap on NFTs that can ever be created (Mac number of admin_minted + user_minted).
    ///
    /// - This is the global maximum supply for the collection.
    /// - `max_supply - released` is implicitly reserved for DAO/admin mints (not available to public users).
    /// - `admin_minted + user_minted` must never exceed this value.
    pub max_supply: u64,

    /// Number of NFTs made available to public users (user-mintable supply).
    ///
    /// - Public/user mints are permitted only while `user_minted < released`.
    /// - The difference `max_supply - released` is reserved for DAO/admin operations (e.g. team mints,
    ///   allocations, airdrops) and cannot be minted by ordinary users.
    /// - Use this field to limit how many NFTs are exposed to the public sale.
    pub released: u64,

    /// Maximum number of NFTs a single user is allowed to mint.
    ///
    /// - Enforced during public/user minting.
    /// - Each unique user wallet cannot exceed this minting cap.
    /// - Prevents whales or bots from exhausting the public supply.
    /// - Use `0` to indicate unlimited user mints (no per-user cap).
    pub max_mint_per_user: u64,

    /// Maximum number of NFTs a single vip user is allowed to mint.
    ///
    /// - Enforced during public/user minting.
    /// - Each unique whitelisted vip user wallet cannot exceed this minting cap.
    /// - Prevents whales or bots vip from exhausting the public supply.
    /// - Use `0` to indicate unlimited vip user mints (no per-user cap).
    pub max_mint_per_vip_user: u64,

    /// Current number of NFTs minted (admin mints).
    ///
    /// - Incremented atomically on each successful admin mint.
    /// - Enforced to never exceed `max_supply - released`.
    pub admin_minted: u64,

    /// Current number of NFTs minted (user mints).
    ///
    /// - Incremented atomically on each successful user mint.
    /// - Enforced to never exceed `released`.
    pub user_minted: u64,

    /// Defines how vesting unlocks are handled for vault redemptions.
    ///
    /// - `VestingMode::None`: No automatic vesting — onchain doesn't restrict NFT burn and escrow refund.
    /// - `VestingMode::Permanent`: Vaults remain locked forever (never redeemable).
    /// - `VestingMode::TimeStamp`: Unlocks automatically after `vesting_unlock_ts`.
    pub vesting_mode: VestingMode,

    /// The UNIX timestamp when escrowed funds become withdrawable for time-based vesting.
    ///
    /// - Used only when `vesting_mode == VestingMode::TimeStamp`.
    /// - If `Clock::get().unix_timestamp >= vesting_unlock_ts`, NFT owners can burn and claim escrow.
    pub vesting_unlock_ts: i64,

    /// The SOL protocol fee (in lamports) charged on each mint.
    ///
    /// - Transferred to the protocol’s treasury wallet.
    /// - Example: `500_000` lamports = 0.0005 SOL.
    pub mint_fee_lamports: u64,

    /// The total mint price per NFT, denominated in the payment mint (e.g. ZDLT).
    ///
    /// - Represents the **full price**, before splitting between vaults and DAO wallets.
    /// - Example: `30_000 * 10^6` = 30,000.000000 ZDLT.
    pub mint_price_total: u64,

    /// The escrowed amount (portion of `mint_price_total`) held in a user-specific vault.
    ///
    /// - Released back to the user after vesting conditions are met.
    pub escrow_amount: u64,

    /// The number of DAO or project wallets that share protocol revenue.
    ///
    /// - Must be ≤ `MAX_REVENUE_WALLETS`.
    /// - Each wallet receives a proportional amount defined in `revenue_shares`.
    pub num_revenue_wallets: u8,

    /// The set of project admin wallets that receive revenue splits from each mint.
    ///
    /// - Indexed 0..`num_revenue_wallets`.
    /// - Each entry corresponds to the same index in `revenue_shares`.
    pub revenue_wallets: [Pubkey; MAX_REVENUE_WALLETS],

    /// The raw (unadjusted) amount in payment tokens each revenue wallet receives.
    ///
    /// - Indexed 0..`num_revenue_wallets`.
    /// - Must sum up (with `escrow_amount`) to ≤ `mint_price_total`.
    pub revenue_shares: [u64; MAX_REVENUE_WALLETS],
}

impl Config {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &[u8; 6] = b"config";
}

impl Config {
    #[inline(always)]
    pub fn init<'a, 'info>(
        bytes: &mut [u8],
        pda_args: InitPdaArgs<'a, 'info>,
        args: InitConfigArgs,
    ) -> ProgramResult {
        Pda::new(pda_args)?.init()?;

        let config = Self::load_mut(bytes)?;
        config.admin = args.admin;
        config.mint = args.mint;
        config.mint_decimals = args.mint_decimals;
        config.max_supply = args.max_supply;
        config.released = args.released;
        config.max_mint_per_user = args.max_mint_per_user;
        config.max_mint_per_vip_user = args.max_mint_per_vip_user;
        config.admin_minted = args.admin_minted;
        config.user_minted = args.user_minted;
        config.vesting_mode = args.vesting_mode;
        config.vesting_unlock_ts = args.vesting_unlock_ts;
        config.mint_fee_lamports = args.mint_fee_lamports;
        config.mint_price_total = args.mint_price_total;
        config.escrow_amount = args.escrow_amount;
        config.num_revenue_wallets = args.num_revenue_wallets;
        config.revenue_wallets = args.revenue_wallets;
        config.revenue_shares = args.revenue_shares;

        Ok(())
    }

    #[inline(always)]
    pub fn init_if_needed<'a, 'info>(
        bytes: &mut [u8],
        pda_args: InitPdaArgs<'a, 'info>,
        args: InitConfigArgs,
    ) -> ProgramResult {
        if UninitializedAccount::check(pda_args.pda).is_ok() {
            Self::init(bytes, pda_args, args)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN {
            msg!("Load config account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() < Self::LEN {
            msg!("Load mut config account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}

impl Config {
    #[inline(always)]
    pub fn is_free_mint_fee(&self) -> bool {
        self.mint_fee_lamports == 0
    }

    #[inline(always)]
    pub fn total_minted(&self) -> u64 {
        self.admin_minted + self.user_minted
    }

    #[inline(always)]
    pub fn admin_supply(&self) -> u64 {
        self.max_supply - self.released
    }

    #[inline(always)]
    pub fn nft_stock_available(&self) -> bool {
        self.total_minted() < self.max_supply
    }

    #[inline(always)]
    pub fn admin_mint_available(&self) -> bool {
        self.admin_minted < self.admin_supply()
    }

    #[inline(always)]
    pub fn user_mint_available(&self) -> bool {
        self.user_minted < self.released
    }

    #[inline(always)]
    pub fn dao_wallet(&self, index: usize) -> Result<&Pubkey, ProgramError> {
        self.revenue_wallets
            .get(index)
            .ok_or(ProgramError::InvalidAccountData)
    }

    #[inline(always)]
    pub fn dao_price(&self, index: usize) -> Result<u64, ProgramError> {
        self.revenue_shares
            .get(index)
            .cloned()
            .ok_or(ProgramError::InvalidAccountData)
    }

    #[inline(always)]
    pub fn need_vault(&self) -> bool {
        self.escrow_amount > 0
    }

    #[inline(always)]
    pub fn allow_tf_to_dao_wallet(&self, index: usize) -> bool {
        let price = self.revenue_shares.get(index).cloned().unwrap_or_default();
        price > 0
    }

    #[inline(always)]
    pub fn increment_admin_minted(&mut self) -> ProgramResult {
        self.admin_minted = self
            .admin_minted
            .checked_add(1)
            .inspect(|_| msg!("Unable to increment config.admin_minted"))
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(())
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
    pub fn check_revenue_wallets(
        mint_price_total: u64,
        escrow_amount: u64,
        num_revenue_wallets: u8,
        revenue_shares: [u64; MAX_REVENUE_WALLETS],
    ) -> ProgramResult {
        let num_wallets = num_revenue_wallets as usize;

        if num_wallets == 0 {
            return Ok(());
        }

        if num_wallets > MAX_REVENUE_WALLETS {
            msg!(
                "Revenue wallets count ({}) exceeds allowed maximum ({})",
                num_wallets,
                MAX_REVENUE_WALLETS
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        let total_revenue_shares: u64 = revenue_shares
            .iter()
            .try_fold(0u64, |acc, &price| {
                acc.checked_add(price)
                    .ok_or(ProgramError::InvalidInstructionData)
            })
            .inspect_err(|_| msg!("Overflow while summing revenue shares"))?;

        let total_mint_price = escrow_amount + total_revenue_shares;

        if total_mint_price != mint_price_total {
            msg!(
                "Inconsistent pricing: expected mint_price_total ({}) = escrow_amount ({}) + total DAO revenue shares ({})",
                mint_price_total,
                escrow_amount,
                total_revenue_shares, 
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn check_nft_royalties(
        num_royalty_recipients: u8,
        royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    ) -> ProgramResult {
        let recipients = num_royalty_recipients as usize;

        if recipients == 0 {
            return Ok(());
        }

        if recipients > MAX_ROYALTY_RECIPIENTS {
            msg!(
                "Too many royalty wallets, max: {}",
                MAX_ROYALTY_RECIPIENTS
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
    pub fn update(&mut self, args: UpdateConfigArgs) {
        self.max_supply = args.max_supply;
        self.released = args.released;
        self.max_mint_per_user = args.max_mint_per_user;
        self.max_mint_per_vip_user = args.max_mint_per_vip_user;
        self.vesting_mode = args.vesting_mode;
        self.vesting_unlock_ts = args.vesting_unlock_ts;
        self.mint_fee_lamports = args.mint_fee_lamports;
        self.mint_price_total = args.mint_price_total;
        self.escrow_amount = args.escrow_amount;
        self.num_revenue_wallets = args.num_revenue_wallets;
        self.revenue_wallets = args.revenue_wallets;
        self.revenue_shares = args.revenue_shares;
    }
}

pub struct InitConfigArgs {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub mint_decimals: u8,
    pub max_supply: u64,
    pub released: u64,
    pub max_mint_per_user: u64,
    pub max_mint_per_vip_user: u64,
    pub admin_minted: u64,
    pub user_minted: u64,
    pub vesting_mode: VestingMode,
    pub vesting_unlock_ts: i64,
    pub mint_fee_lamports: u64,
    pub mint_price_total: u64,
    pub escrow_amount: u64,
    pub num_revenue_wallets: u8,
    pub revenue_wallets: [Pubkey; MAX_REVENUE_WALLETS],
    pub revenue_shares: [u64; MAX_REVENUE_WALLETS],
}

pub struct UpdateConfigArgs {
    pub max_supply: u64,
    pub released: u64,
    pub max_mint_per_user: u64,
    pub max_mint_per_vip_user: u64,
    pub vesting_mode: VestingMode,
    pub vesting_unlock_ts: i64,
    pub mint_fee_lamports: u64,
    pub mint_price_total: u64,
    pub escrow_amount: u64,
    pub num_revenue_wallets: u8,
    pub revenue_wallets: [Pubkey; MAX_REVENUE_WALLETS],
    pub revenue_shares: [u64; MAX_REVENUE_WALLETS],
}

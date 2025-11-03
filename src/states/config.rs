use bytemuck::{Pod, Zeroable};
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Config {
    /// The authority that can update config or perform admin actions.
    /// Must match the signer in `update_config_v1`.
    pub authority: Pubkey,

    /// Maximum total NFTs that can ever be minted.
    /// Enforced on-chain. Once reached, minting stops permanently.
    pub max_supply: u64,

    /// Number of NFTs reserved for whitelist (WL) phase.
    /// When `supply_minted < released`, WL rules apply (e.g. Merkle proof).
    /// After this, public mint begins.
    pub released: u64,

    /// Price per NFT in ZDLT tokens (raw amount, not decimal-adjusted).
    /// Example: `30_000 * 10^6` = 30,000.000000 ZDLT.
    pub price: u64,

    /// Current number of NFTs minted.
    /// Incremented atomically on each successful mint.
    /// Cannot exceed `max_supply`.
    pub supply_minted: u64,

    /// Unix timestamp when escrowed SPL token mint (e.g. ZDLT) becomes withdrawable.
    /// Used in `burn_and_refund_v1` to enforce vesting.
    /// If `clock.unix_timestamp >= vesting_end_ts`, user can burn NFT and claim tokens.
    pub vesting_end_ts: i64,

    /// The SPL token mint (e.g. ZDLT) used for payment and escrow.
    /// Must match `token_mint` in `mint_and_vault_v1`.
    pub mint: Pubkey,

    /// Number of decimals in the payment token (e.g. 6 for ZDLT).
    /// Used to validate token transfer amounts.
    pub mint_decimals: u8,

    /// SOL protocol fee in lamports, paid by minter on every mint.
    /// Transferred to `protocol_wallet` account.
    /// Example: `500_000` = 0.0005 SOL.
    pub protocol_fee_lamports: u64,
}

impl Config {
    pub const LEN: usize = size_of::<Self>();

    pub const SEED: &[u8; 6] = b"config";

    #[inline(always)]
    pub fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            msg!("Load config account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        let config: &Self = bytemuck::try_from_bytes(&data[..Self::LEN])
            .inspect_err(|_| msg!("Invalid loaded config data"))
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(config)
    }

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            msg!("Load mut config account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        let config: &mut Self = bytemuck::try_from_bytes_mut(&mut data[..Self::LEN])
            .inspect_err(|_| msg!("Invalid loaded mutable config data"))
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(config)
    }

    #[inline(always)]
    pub fn init(data: &mut [u8], cfg: &Self) -> Result<(), ProgramError> {
        if data.len() < Self::LEN {
            msg!("Init config account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }
        let src = bytemuck::bytes_of(cfg);
        data[..Self::LEN].copy_from_slice(src);
        Ok(())
    }

    #[inline(always)]
    pub fn increment_minted(&mut self) -> Result<(), ProgramError> {
        self.supply_minted = self
            .supply_minted
            .checked_add(1)
            .inspect(|_| msg!("Unable to increment config.minted"))
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(())
    }
}

use bytemuck::{Pod, Zeroable};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vault {
    /// The user who paid for and owns the escrowed tokens.
    /// Must match `payer` in `mint_and_vault_v1`.
    pub owner: Pubkey,

    /// The MPL Core NFT asset (mint) associated with this escrow.
    /// Used to match NFT on burn.
    pub nft: Pubkey,

    /// Amount of ZDLT tokens escrowed (raw, matches `config.price`).
    /// Returned to `owner` on successful burn + vesting.
    pub amount: u64,

    /// Boolean flag: `1` = tokens withdrawn, `0` = still locked.
    /// Set in `burn_and_refund_v1` after transfer.
    pub is_unlocked: u8,

    /// PDA bump seed for `["vault", config_pda]`.
    /// Stored for replay safety and future use.
    pub bump: [u8; 1],
}

impl Vault {
    pub const LEN: usize = size_of::<Pubkey>()
        + size_of::<Pubkey>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<bool>()
        + size_of::<[u8; 1]>();

    pub const SEED: &[u8; 5] = b"vault";

    pub fn new(owner: Pubkey, nft: Pubkey, amount: u64, is_unlocked: bool, bump: [u8; 1]) -> Self {
        Self {
            owner,
            nft,
            amount,
            is_unlocked: if is_unlocked { 1 } else { 0 },
            bump,
        }
    }

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let minted: &mut Self = bytemuck::try_from_bytes_mut(&mut data[..Self::LEN])
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(minted)
    }

    #[inline(always)]
    pub fn init(data: &mut [u8], cfg: &Self) -> Result<(), ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        data.copy_from_slice(bytemuck::bytes_of(cfg));
        Ok(())
    }

    #[inline(always)]
    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked == 1
    }

    #[inline(always)]
    pub fn set_unlocked(&mut self, is_unlocked: bool) {
        self.is_unlocked = if is_unlocked { 1 } else { 0 };
    }
}

use bytemuck::{Pod, Zeroable};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

pub const VAULT_SEED: &[u8; 5] = b"vault";

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vault {
    pub owner: Pubkey,
    pub nft: Pubkey,
    pub amount: u64,
    pub is_unlocked: u8,
    pub bump: [u8; 1],
}

impl Vault {
    pub const LEN: usize = size_of::<Pubkey>()
        + size_of::<Pubkey>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<bool>()
        + size_of::<[u8; 1]>();

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

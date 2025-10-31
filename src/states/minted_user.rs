use bytemuck::{Pod, Zeroable};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

pub const MINTED_USER_SEED: &[u8; 11] = b"minted_user";

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MintedUser {
    pub owner: Pubkey,
    pub minted: u8,
}

impl MintedUser {
    pub const LEN: usize = size_of::<Pubkey>() + size_of::<bool>();

    pub fn new(owner: Pubkey, minted: bool) -> Self {
        Self {
            owner,
            minted: if minted { 1 } else { 0 },
        }
    }

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        bytemuck::try_from_bytes_mut(&mut data[..Self::LEN])
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    #[inline(always)]
    pub fn init(data: &mut [u8], minted_user: &Self) -> Result<(), ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        data.copy_from_slice(bytemuck::bytes_of(minted_user));
        Ok(())
    }

    #[inline(always)]
    pub fn is_minted(&self) -> bool {
        self.minted == 1
    }

    #[inline(always)]
    pub fn set_minted(&mut self, minted: bool) {
        self.minted = if minted { 1 } else { 0 };
    }
}

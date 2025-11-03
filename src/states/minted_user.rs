use bytemuck::{Pod, Zeroable};
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MintedUser {
    /// The wallet that owns this mint record.
    /// Must match the `payer` in `mint_and_vault_v1`.
    pub owner: Pubkey,

    /// Boolean flag: `1` = already minted, `0` = not minted.
    /// Set to `1` atomically during `mint_and_vault_v1`.
    /// Checked before allowing mint.
    pub minted: u8,
}

impl MintedUser {
    pub const LEN: usize = size_of::<Self>();

    pub const SEED: &[u8; 11] = b"minted_user";

    pub fn new(owner: Pubkey, minted: bool) -> Self {
        Self {
            owner,
            minted: if minted { 1 } else { 0 },
        }
    }

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            msg!("Load mut minted user account data length wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        bytemuck::try_from_bytes_mut(&mut data[..Self::LEN])
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    #[inline(always)]
    pub fn init(data: &mut [u8], minted_user: &Self) -> Result<(), ProgramError> {
        if data.len() < Self::LEN {
            msg!("Init minted user account data length wrong");
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

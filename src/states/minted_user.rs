use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh1::try_from_slice_unchecked, program_error::ProgramError, pubkey::Pubkey,
};

pub const MINTED_USER_SEED: &[u8; 11] = b"minted_user";

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MintedUser {
    pub owner: Pubkey,
    pub minted: bool,
}

impl MintedUser {
    pub const LEN: usize = size_of::<Pubkey>() + size_of::<bool>();

    #[inline(always)]
    pub fn load(data: &[u8]) -> Result<Self, ProgramError> {
        let result: Self = try_from_slice_unchecked(data)?;
        Ok(result)
    }
}

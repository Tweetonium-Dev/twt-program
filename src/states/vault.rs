use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{borsh1::try_from_slice_unchecked, program_error::ProgramError, pubkey::Pubkey};

pub const VAULT_SEED: &[u8; 5] = b"vault";

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Vault {
    pub owner: Pubkey,
    pub nft: Pubkey,
    pub amount: u64,
    pub is_unlocked: bool,
    pub bump: [u8; 1],
}

impl Vault {
    pub const LEN: usize = size_of::<Pubkey>()
        + size_of::<Pubkey>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<bool>()
        + size_of::<[u8; 1]>();

    #[inline(always)]
    pub fn load(data: &[u8]) -> Result<Self, ProgramError> {
        let result: Self = try_from_slice_unchecked(data)?;
        Ok(result)
    }
}

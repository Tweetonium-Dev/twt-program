use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{borsh1::try_from_slice_unchecked, program_error::ProgramError, pubkey::Pubkey};

pub const CONFIG_SEED: &[u8; 6] = b"config";

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Config {
    pub authority: Pubkey,
    pub max_supply: u64,
    pub released: u64,
    pub price: u64,
    pub supply_minted: u64,
    pub vesting_end_ts: i64,
    pub merkle_root: Pubkey,
    pub mint: Pubkey,
}

impl Config {
    pub const LEN: usize = size_of::<Pubkey>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<i64>()
        + size_of::<Pubkey>()
        + size_of::<Pubkey>();

    #[inline(always)]
    pub fn load(data: &[u8]) -> Result<Self, ProgramError> {
        let result: Self = try_from_slice_unchecked(data)?;
        Ok(result)
    }
}

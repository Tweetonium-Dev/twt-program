use bytemuck::{Pod, Zeroable};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

pub const CONFIG_SEED: &[u8; 6] = b"config";

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Config {
    pub authority: Pubkey,
    pub max_supply: u64,
    pub released: u64,
    pub price: u64,
    pub supply_minted: u64,
    pub vesting_end_ts: i64,
    pub merkle_root: Pubkey,
    pub mint: Pubkey,
    pub mint_decimals: u8,
}

impl Config {
    pub const LEN: usize = size_of::<Pubkey>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<i64>()
        + size_of::<Pubkey>()
        + size_of::<Pubkey>()
        + size_of::<u8>();

    #[inline(always)]
    pub fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let config: &Self = bytemuck::try_from_bytes(&data[..Self::LEN])
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(config)
    }

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let config: &mut Self = bytemuck::try_from_bytes_mut(&mut data[..Self::LEN])
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(config)
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
    pub fn increment_minted(&mut self) -> Result<(), ProgramError> {
        self.supply_minted = self
            .supply_minted
            .checked_add(1)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(())
    }
}

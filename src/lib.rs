use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    instructions::{
        BurnAndRefundV1, ForceUnlockVestingV1, InitConfigV1, InitConfigV1InstructionData,
        InitTraitV1, InitTraitV1InstructionData, MintAdminV1, MintAdminV1InstructionData,
        MintTraitV1, MintTraitV1InstructionData, MintUserV1, MintUserV1InstructionData, MintVipV1,
        MintVipV1InstructionData, UpdateConfigV1, UpdateConfigV1InstructionData, UpdateNftV1,
        UpdateNftV1InstructionData, UpdateTraitV1, UpdateTraitV1InstructionData,
    },
    utils::ProcessInstruction,
};

pub mod instructions;
pub mod states;
pub mod utils;

declare_id!("TWTfEU1tgnaErUq4BetvcskjrkV1Hz5K3pgS4ezzytt");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((0, data)) => process_init_config(program_id, accounts, data),
        Some((1, data)) => process_update_config(program_id, accounts, data),
        Some((2, data)) => process_mint_admin(program_id, accounts, data),
        Some((3, data)) => process_mint_user(program_id, accounts, data),
        Some((4, data)) => process_mint_vip(program_id, accounts, data),
        Some((5, data)) => process_init_trait(program_id, accounts, data),
        Some((6, data)) => process_update_trait(program_id, accounts, data),
        Some((7, data)) => process_mint_trait(program_id, accounts, data),
        Some((8, data)) => process_update_nft(program_id, accounts, data),
        Some((9, _)) => process_burn_nft(program_id, accounts),
        Some((10, _)) => process_force_unlock_vesting(program_id, accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[inline(never)]
fn process_init_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Initialize Config");
    let data = InitConfigV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    InitConfigV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_update_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Update Config");
    let data = UpdateConfigV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    UpdateConfigV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_mint_admin(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Mint Admin");
    let data = MintAdminV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    MintAdminV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_mint_user(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Mint User");
    let data = MintUserV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    MintUserV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_mint_vip(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Mint Vip");
    let data = MintVipV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    MintVipV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_init_trait(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Initialize Trait");
    let data = InitTraitV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    InitTraitV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_update_trait(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Update Trait");
    let data = UpdateTraitV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    UpdateTraitV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_mint_trait(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Mint Trait");
    let data = MintTraitV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    MintTraitV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_update_nft(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Update NFT");
    let data = UpdateNftV1InstructionData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    UpdateNftV1::try_from((accounts, data, program_id))?.process()
}

#[inline(never)]
fn process_burn_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Burn NFT");
    BurnAndRefundV1::try_from((accounts, program_id))?.process()
}

#[inline(never)]
fn process_force_unlock_vesting(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Force Unlock Vesting");
    ForceUnlockVestingV1::try_from((accounts, program_id))?.process()
}

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    instructions::{
        BurnAndRefundV1, ForceUnlockVestingV1, InitConfigV1, InitTraitV1, MintAdminV1, MintTraitV1,
        MintUserV1, MintVipV1, TweetoniumInstruction, UpdateConfigV1, UpdateNftV1, UpdateTraitV1,
    },
    utils::ProcessInstruction,
};

mod instructions;
mod states;
mod utils;

declare_id!("TWTfEU1tgnaErUq4BetvcskjrkV1Hz5K3pgS4ezzytt");

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TweetoniumInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TweetoniumInstruction::InitConfigV1(data) => {
            msg!("InitializeConfig");
            InitConfigV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::UpdateConfigV1(data) => {
            msg!("UpdateConfig");
            UpdateConfigV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::MintAdminV1(data) => {
            msg!("MintAdmin");
            MintAdminV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::MintUserV1(data) => {
            msg!("MintUser");
            MintUserV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::MintVipV1(data) => {
            msg!("MintVip");
            MintVipV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::InitTraitV1(data) => {
            msg!("InitTrait");
            InitTraitV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::UpdateTraitV1(data) => {
            msg!("UpdateTrait");
            UpdateTraitV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::MintTraitV1(data) => {
            msg!("MintTrait");
            MintTraitV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::UpdateNftV1(data) => {
            msg!("UpdateNft");
            UpdateNftV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::BurnAndRefundV1 => {
            msg!("BurnAndRefund");
            BurnAndRefundV1::try_from((accounts, program_id))?.process()
        }
        TweetoniumInstruction::ForceUnlockVestingV1 => {
            msg!("ForceUnlockVesting");
            ForceUnlockVestingV1::try_from((accounts, program_id))?.process()
        }
    }
}

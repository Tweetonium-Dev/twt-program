use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    instructions::{
        BurnAndRefundV1, ForceUnlockVestingV1, InitConfigV1, MintAndVaultV1, TweetoniumInstruction,
        UpdateNftV1,
    },
    utils::ProcessInstruction,
};

mod instructions;
mod states;
mod utils;

declare_id!("GHSZjEbYB9ZCSAid6qdgCEMZHB1P6MK9dCZ5yescwrrD");

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
            msg!("Instruction: InitializeConfig");
            InitConfigV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::MintAndVaultV1(data) => {
            msg!("Instruction: MintAndVault");
            MintAndVaultV1::try_from((accounts, data, program_id))?.process()
        }
        TweetoniumInstruction::UpdateNftV1(data) => {
            msg!("Instruction: UpdateNft");
            UpdateNftV1::try_from((accounts, data))?.process()
        }
        TweetoniumInstruction::BurnAndRefundV1 => {
            msg!("Instruction: BurnAndRefund");
            BurnAndRefundV1::try_from((accounts, program_id))?.process()
        }
        TweetoniumInstruction::ForceUnlockVestingV1 => {
            msg!("Instruction: ForceUnlockVesting");
            ForceUnlockVestingV1::try_from(accounts)?.process()
        }
    }
}

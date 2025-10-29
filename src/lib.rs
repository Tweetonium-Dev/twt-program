#![cfg_attr(not(check_cfg), allow(unexpected_cfgs))]
#![cfg(check_cfg)]
#![check_cfg(
    feature = "custom-heap",
    feature = "custom-panic",
    target_os = "solana"
)]

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    instructions::{BurnAndRefundV1, InitConfigV1, Instructions, MintAndVaultV1},
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
    let instruction = Instructions::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        Instructions::InitConfigV1(data) => {
            msg!("Instruction: InitializeConfig");
            InitConfigV1::try_from((accounts, data, program_id))?.process()
        }
        Instructions::MintAndVaultV1(data) => {
            msg!("Instruction: MintAndVault");
            MintAndVaultV1::try_from((accounts, data, program_id))?.process()
        }
        Instructions::BurnAndRefundV1 => {
            msg!("Instruction: BurnAndRefund");
            BurnAndRefundV1::try_from((accounts, program_id))?.process()
        }
    }
}

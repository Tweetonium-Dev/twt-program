use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar
};
use solana_system_interface::instruction as system_instruction;

use crate::{
    states::{Config, CONFIG_SEED},
    utils::{
        AccountCheck, ProcessInstruction, SignerAccount, SystemAccount,
        WritableAccount,
    },
};

#[derive(Debug)]
pub struct InitConfigV1Accounts<'a, 'info> {
    pub authority: &'a AccountInfo<'info>,
    pub config: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for InitConfigV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, config, mint, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;
        WritableAccount::check(config)?;
        SystemAccount::check(system_program)?;

        Ok(Self {
            authority,
            config,
            mint,
            system_program,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitConfigV1InstructionData {
    pub max_supply: u64,
    pub released: u64,
    pub price: u64,
    pub vesting_end_ts: i64,
    pub merkle_root: Pubkey,
}

#[derive(Debug)]
pub struct InitConfigV1<'a, 'info> {
    pub accounts: InitConfigV1Accounts<'a, 'info>,
    pub instruction_data: InitConfigV1InstructionData,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        InitConfigV1InstructionData,
        &'a Pubkey,
    )> for InitConfigV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            InitConfigV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = InitConfigV1Accounts::try_from(accounts)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
        })
    }
}

impl<'a, 'info> ProcessInstruction for InitConfigV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let authority = self.accounts.authority;
        let config = self.accounts.config;
        let mint = self.accounts.mint;
        let system_program = self.accounts.system_program;

        let (expected_pda, bump) = Pubkey::find_program_address(
            &[CONFIG_SEED, authority.key.as_ref()],
            self.program_id,
        );

        if expected_pda != *config.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if config.data_is_empty() {
            let rent = Rent::get()?;
            let space = Config::LEN;
            let lamports = rent.minimum_balance(space);

            invoke_signed(
                &system_instruction::create_account(
                    authority.key,
                    config.key,
                    lamports,
                    space as u64,
                    self.program_id,
                ),
                &[authority.clone(), config.clone(), system_program.clone()],
                &[&[CONFIG_SEED, authority.key.as_ref(), &[bump]]],
            )?;
        }

        let cfg = Config {
            authority: *authority.key,
            max_supply: self.instruction_data.max_supply,
            released: self.instruction_data.released,
            price: self.instruction_data.price,
            supply_minted: 0,
            vesting_end_ts: self.instruction_data.vesting_end_ts,
            merkle_root: self.instruction_data.merkle_root,
            mint: *mint.key,
        };

        cfg.serialize(&mut &mut config.data.borrow_mut()[..])?;

        Ok(())
    }
}

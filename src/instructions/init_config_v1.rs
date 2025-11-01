use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::Config,
    utils::{
        AccountCheck, MintAccount, Pda, ProcessInstruction, SignerAccount, SystemAccount,
        TokenProgram, WritableAccount,
    },
};

#[derive(Debug)]
pub struct InitConfigV1Accounts<'a, 'info> {
    /// Authority that will control config updates (e.g. admin wallet).
    /// Must be a signer.
    pub authority: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "config"]` — stores `Config` struct.
    /// Must be uninitialized, writable, owned by this program.
    pub config_pda: &'a AccountInfo<'info>,

    /// Token mint (fungible token used for minting/refunding e.g. ZDLT).
    /// Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022.
    pub mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy) or Token-2022 Program.
    /// Must match the mint's owner.
    pub token_program: &'a AccountInfo<'info>,

    /// System program — required for PDA creation and rent.
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for InitConfigV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, config_pda, mint, token_program, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;
        WritableAccount::check(config_pda)?;
        MintAccount::check(mint)?;
        SystemAccount::check(system_program)?;

        Ok(Self {
            authority,
            config_pda,
            mint,
            token_program,
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
    pub protocol_fee_lamports: u64,
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
        let config_pda = self.accounts.config_pda;
        let mint = self.accounts.mint;
        let token_program = self.accounts.token_program;
        let system_program = self.accounts.system_program;

        Pda::new(
            authority,
            config_pda,
            system_program,
            &[Config::SEED, authority.key.as_ref()],
            Config::LEN,
            self.program_id,
            self.program_id,
        )?
        .init_if_needed()?;

        let decimals = TokenProgram::get_decimal(mint, token_program)?;

        let cfg = Config {
            authority: *authority.key,
            max_supply: self.instruction_data.max_supply,
            released: self.instruction_data.released,
            price: self.instruction_data.price,
            supply_minted: 0,
            vesting_end_ts: self.instruction_data.vesting_end_ts,
            merkle_root: self.instruction_data.merkle_root,
            mint: *mint.key,
            mint_decimals: decimals,
            protocol_fee_lamports: self.instruction_data.protocol_fee_lamports,
        };

        Config::init(&mut config_pda.data.borrow_mut()[..], &cfg)?;

        Ok(())
    }
}

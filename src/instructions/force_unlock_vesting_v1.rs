use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::Config,
    utils::{
        AccountCheck, ConfigAccount, MintAccount, Pda, ProcessInstruction, SignerAccount,
        WritableAccount,
    },
};

#[derive(Debug)]
pub struct ForceUnlockVestingV1Accounts<'a, 'info> {
    /// The config authority — must sign.
    /// Must match `config.authority`.
    pub authority: &'a AccountInfo<'info>,

    /// PDA: `[program_id, token_mint, "config"]` — stores global config.
    /// Must be writable.
    pub config_pda: &'a AccountInfo<'info>,

    /// Token mint (fungible token used for minting/refunding e.g. ZDLT).
    /// Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022.
    pub token_mint: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for ForceUnlockVestingV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, config_pda, token_mint] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;

        WritableAccount::check(config_pda)?;

        MintAccount::check(token_mint)?;
        ConfigAccount::check(config_pda)?;

        Ok(Self {
            authority,
            config_pda,
            token_mint,
        })
    }
}

#[derive(Debug)]
pub struct ForceUnlockVestingV1<'a, 'info> {
    pub accounts: ForceUnlockVestingV1Accounts<'a, 'info>,
}

impl<'a, 'info> ForceUnlockVestingV1<'a, 'info> {
    fn check_authority(&self, config: &Config) -> ProgramResult {
        if config.authority != *self.accounts.authority.key {
            msg!("Only config authority can force unlock vesting");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    fn unlock_vesting(&self, config: &mut Config) -> ProgramResult {
        let now = Clock::get()?.unix_timestamp;

        if config.vesting_end_ts <= now {
            msg!("Vesting already unlocked or in the past");
            return Err(ProgramError::Custom(1));
        }

        let old_ts = config.vesting_end_ts;
        config.vesting_end_ts = now;

        msg!(
            "ForceUnlockVesting: vesting unlocked early. Was {} → now {}",
            old_ts,
            now
        );

        Ok(())
    }
}

impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)>
    for ForceUnlockVestingV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        let accounts = ForceUnlockVestingV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.config_pda,
            &[Config::SEED, accounts.token_mint.key.as_ref()],
            program_id,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a, 'info> ProcessInstruction for ForceUnlockVestingV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut config_data = self.accounts.config_pda.data.borrow_mut();
        let config = Config::load_mut(&mut config_data)?;

        self.check_authority(config)?;
        self.unlock_vesting(config)?;

        Ok(())
    }
}

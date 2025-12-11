use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::{ProjectV1, VestingMode},
    utils::{
        AccountCheck, MintAccount, Pda, ProcessInstruction, ProjectAccount, SignerAccount,
        WritableAccount,
    },
};

#[derive(Debug)]
pub struct ForceUnlockVestingV1Accounts<'a, 'info> {
    /// The project authority — must sign.
    /// Must match `project.admin`.
    pub admin: &'a AccountInfo<'info>,

    /// PDA: `["project_v1", nft_collection, token_mint, program_id]`.
    /// Must be writable.
    pub project_pda: &'a AccountInfo<'info>,

    /// Token mint (fungible token used for minting/refunding e.g. ZDLT).
    /// Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022.
    pub token_mint: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for ForceUnlockVestingV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [admin, project_pda, token_mint, nft_collection] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(admin)?;

        WritableAccount::check(project_pda)?;
        WritableAccount::check(nft_collection)?;

        MintAccount::check(token_mint)?;
        ProjectAccount::check(project_pda)?;

        Ok(Self {
            admin,
            project_pda,
            token_mint,
            nft_collection,
        })
    }
}

#[derive(Debug)]
pub struct ForceUnlockVestingV1<'a, 'info> {
    pub accounts: ForceUnlockVestingV1Accounts<'a, 'info>,
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
            accounts.project_pda,
            &[
                ProjectV1::SEED,
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a, 'info> ProcessInstruction for ForceUnlockVestingV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut config_data = self.accounts.project_pda.data.borrow_mut();
        let config = ProjectV1::load_mut(&mut config_data)?;

        self.check_vesting(config)?;
        self.unlock_vesting(config)
    }
}

impl<'a, 'info> ForceUnlockVestingV1<'a, 'info> {
    fn check_vesting(&self, config: &ProjectV1) -> ProgramResult {
        if config.admin != *self.accounts.admin.key {
            msg!("Unauthorized: only the config authority may trigger vesting unlocks.");
            return Err(ProgramError::IllegalOwner);
        }

        match config.vesting_mode {
            VestingMode::None => {
                msg!("Vesting unlock denied: vesting mode is disabled (None).");
                Err(ProgramError::InvalidInstructionData)
            }
            VestingMode::Permanent => {
                msg!("Vesting unlock denied: this vault is permanently locked.");
                Err(ProgramError::Immutable)
            }
            VestingMode::TimeStamp => Ok(()),
        }
    }

    fn unlock_vesting(&self, config: &mut ProjectV1) -> ProgramResult {
        let now = Clock::get()?.unix_timestamp;

        if config.vesting_unlock_ts <= now {
            msg!("Vesting already unlocked");
            return Ok(());
        }

        let old_ts = config.vesting_unlock_ts;
        config.vesting_unlock_ts = now;

        msg!(
            "ForceUnlockVesting: vesting unlocked early. Was {} → now {}",
            old_ts,
            now
        );

        Ok(())
    }
}

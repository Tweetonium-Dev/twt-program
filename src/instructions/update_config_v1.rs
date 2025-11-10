use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{Config, UpdateConfigArgs, VestingMode, MAX_REVENUE_WALLETS, MAX_ROYALTY_RECIPIENTS},
    utils::{
        AccountCheck, MintAccount, MplCoreProgram, Pda, ProcessInstruction, SignerAccount,
        SystemProgram, UpdateMplCoreCollectionAccounts, UpdateMplCoreCollectionArgs,
        WritableAccount,
    },
};

#[derive(Debug)]
pub struct UpdateConfigV1Accounts<'a, 'info> {
    /// Authority that will control config updates (e.g. admin wallet).
    /// Must be a signer.
    pub authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be initialized before config creation via `CreateV1CpiBuilder`.
    /// Used as part of the config PDA seeds: `[program_id, token_mint, collection.key.as_ref()]`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// PDA: `[program_id, token_mint, "config"]` — stores `Config` struct.
    /// Must be uninitialized, writable, owned by this program.
    pub config_pda: &'a AccountInfo<'info>,

    /// Token mint (fungible token used for minting/refunding e.g. ZDLT).
    /// Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022.
    pub token_mint: &'a AccountInfo<'info>,

    /// System program — required for PDA creation and rent.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for UpdateConfigV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, nft_collection, config_pda, token_mint, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;

        WritableAccount::check(nft_collection)?;
        WritableAccount::check(config_pda)?;

        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        Ok(Self {
            authority,
            nft_collection,
            config_pda,
            token_mint,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct UpdateConfigV1InstructionData {
    pub max_supply: u64,
    pub released: u64,
    pub max_mint_per_user: u64,
    pub max_mint_per_vip_user: u64,
    pub vesting_mode: VestingMode,
    pub vesting_unlock_ts: i64,
    pub mint_fee_lamports: u64,
    pub mint_price_total: u64,
    pub escrow_amount: u64,
    pub num_revenue_wallets: u8,
    pub revenue_wallets: [Pubkey; 5],
    pub revenue_shares: [u64; 5],
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; 5],
    pub royalty_shares_bps: [u16; 5],
    pub collection_name: String,
    pub collection_uri: String,
}

#[derive(Debug)]
pub struct UpdateConfigV1<'a, 'info> {
    pub accounts: UpdateConfigV1Accounts<'a, 'info>,
    pub instruction_data: UpdateConfigV1InstructionData,
}

impl<'a, 'info> UpdateConfigV1<'a, 'info> {
    fn check_revenue_wallets(&self) -> ProgramResult {
        let num_wallets = self.instruction_data.num_revenue_wallets as usize;

        if num_wallets == 0 {
            return Ok(());
        }

        if num_wallets > MAX_REVENUE_WALLETS {
            msg!(
                "Invalid configuration: DAO wallet count ({}) exceeds allowed maximum ({})",
                num_wallets,
                MAX_REVENUE_WALLETS
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        let total_dao_price: u64 = self
            .instruction_data
            .revenue_shares
            .iter()
            .try_fold(0u64, |acc, &price| {
                acc.checked_add(price)
                    .ok_or(ProgramError::InvalidInstructionData)
            })
            .inspect_err(|_| msg!("Overflow while summing DAO revenue shares"))?;

        let total_dao_shares = self.instruction_data.escrow_amount + total_dao_price;

        if total_dao_shares != self.instruction_data.mint_price_total {
            msg!(
                "Inconsistent pricing: expected mint_price_total ({}) = escrow_amount ({}) + total DAO shares ({})",
                self.instruction_data.mint_price_total,
                self.instruction_data.escrow_amount,
                total_dao_shares
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(())
    }

    fn check_nft_royalties(&self) -> ProgramResult {
        let recipients = self.instruction_data.num_royalty_recipients as usize;

        if recipients == 0 {
            return Ok(());
        }

        if recipients > MAX_ROYALTY_RECIPIENTS {
            msg!("Too many royalty wallets, max: {}", MAX_ROYALTY_RECIPIENTS);
            return Err(ProgramError::InvalidInstructionData);
        }

        let total_bps: u16 = self
            .instruction_data
            .royalty_shares_bps
            .iter()
            .try_fold(0u16, |acc, &price| {
                acc.checked_add(price)
                    .ok_or(ProgramError::InvalidInstructionData)
            })
            .inspect_err(|_| msg!("Failed to sum total bps"))?;

        if total_bps > 10_000 {
            msg!("Total royalty BPS exceeds 100% (10_000)");
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(())
    }

    fn update_collection(&self) -> ProgramResult {
        MplCoreProgram::update_collection(
            UpdateMplCoreCollectionAccounts {
                collection: self.accounts.nft_collection,
                authority: self.accounts.authority,
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            UpdateMplCoreCollectionArgs {
                num_royalty_recipients: self.instruction_data.num_royalty_recipients,
                royalty_recipients: self.instruction_data.royalty_recipients,
                royalty_shares_bps: self.instruction_data.royalty_shares_bps,
                name: self.instruction_data.collection_name.clone(),
                uri: self.instruction_data.collection_uri.clone(),
            },
        )
    }

    fn update_config(&self) -> ProgramResult {
        let mut config_data = self.accounts.config_pda.try_borrow_mut_data()?;
        let config = Config::load_mut(config_data.as_mut())?;

        if config.admin != *self.accounts.authority.key {
            msg!("Unauthorized authority for config update");
            return Err(ProgramError::InvalidAccountData);
        }

        config.update(UpdateConfigArgs {
            max_supply: self.instruction_data.max_supply,
            released: self.instruction_data.released,
            max_mint_per_user: self.instruction_data.max_mint_per_user,
            max_mint_per_vip_user: self.instruction_data.max_mint_per_vip_user,
            vesting_mode: self.instruction_data.vesting_mode,
            vesting_unlock_ts: self.instruction_data.vesting_unlock_ts,
            mint_fee_lamports: self.instruction_data.mint_fee_lamports,
            mint_price_total: self.instruction_data.mint_price_total,
            escrow_amount: self.instruction_data.escrow_amount,
            num_revenue_wallets: self.instruction_data.num_revenue_wallets,
            revenue_wallets: self.instruction_data.revenue_wallets,
            revenue_shares: self.instruction_data.revenue_shares,
        });

        Ok(())
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        UpdateConfigV1InstructionData,
        &'a Pubkey,
    )> for UpdateConfigV1<'a, 'info>
{
    type Error = ProgramError;
    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            UpdateConfigV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = UpdateConfigV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.config_pda,
            &[
                Config::SEED,
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a, 'info> ProcessInstruction for UpdateConfigV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.check_revenue_wallets()?;
        self.check_nft_royalties()?;
        self.update_collection()?;
        self.update_config()
    }
}

use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::{ConfigV1, NftAuthorityV1, VaultV1, VestingMode},
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck,
        BurnMplCoreAssetAccounts, ConfigAccount, MintAccount, MplCoreProgram, Pda,
        ProcessInstruction, SignerAccount, SystemProgram, TokenProgram, TokenTransferAccounts,
        TokenTransferArgs, VaultAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct BurnAndRefundV1Accounts<'a, 'info> {
    /// NFT owner — must sign to burn.
    /// Must be owner of `nft_token_account`.
    pub payer: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "nft_authority"]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be initialized before config creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// NFT asset — must be burned.
    /// Must be valid MPL Core asset.
    pub nft_asset: &'a AccountInfo<'info>,

    /// PDA: `[program_id, token_mint, nft_collection, nft_asset, "vault"]` — escrow state.
    /// Must be readable.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Vault's ATA — source of refund token_mint.
    /// Must be writable, owned by `token_program`.
    pub vault_ata: &'a AccountInfo<'info>,

    /// User's ATA — receives refund.
    /// Must be writable, owned by `token_program`.
    pub payer_ata: &'a AccountInfo<'info>,

    /// PDA: `[program_id, token_mint, nft_collection, "config"]` — for price/refund logic.
    /// Must be readable.
    pub config_pda: &'a AccountInfo<'info>,

    /// Token mint — must match config (e.g. ZDLT.
    /// Must be valid mint.
    pub token_mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_asset.owner`.
    pub token_program: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for BurnAndRefundV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [payer, nft_authority, nft_collection, nft_asset, vault_pda, vault_ata, payer_ata, config_pda, token_mint, token_program, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;

        WritableAccount::check(nft_collection)?;
        WritableAccount::check(nft_asset)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;

        VaultAccount::check(vault_pda)?;
        ConfigAccount::check(config_pda)?;
        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        AssociatedTokenAccount::check(vault_ata, vault_pda.key, token_mint.key, token_program.key)?;
        AssociatedTokenAccount::check(payer_ata, payer.key, token_mint.key, token_program.key)?;

        Ok(Self {
            payer,
            nft_authority,
            nft_collection,
            nft_asset,
            vault_pda,
            vault_ata,
            payer_ata,
            config_pda,
            token_mint,
            token_program,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug)]
pub struct BurnAndRefundV1<'a, 'info> {
    pub accounts: BurnAndRefundV1Accounts<'a, 'info>,
    pub nft_authority_bump: u8,
    pub vault_bump: u8,
}

impl<'a, 'info> BurnAndRefundV1<'a, 'info> {
    fn check_vesting(&self, config: &ConfigV1, vault: &VaultV1) -> ProgramResult {
        let clock = Clock::get()?;
        let asset = MplCoreProgram::get_asset(self.accounts.nft_asset)?;

        if asset.base.owner != *self.accounts.payer.key {
            msg!(
                "Payer is not the current owner of the NFT. Owner: {}, Payer: {}",
                asset.base.owner,
                self.accounts.payer.key,
            );
            return Err(ProgramError::IllegalOwner);
        }

        if vault.is_unlocked() {
            msg!("Vault has already been refunded or unlocked.");
            return Err(ProgramError::InvalidAccountData);
        }

        match config.vesting_mode {
            VestingMode::None => Ok(()),
            VestingMode::Permanent => {
                msg!("This vault is permanently locked — burn and refund not allowed.");
                Err(ProgramError::Immutable)
            }
            VestingMode::TimeStamp => {
                if clock.unix_timestamp < config.vesting_unlock_ts {
                    msg!(
                        "Vesting not yet complete: current ts={} < unlock ts={}",
                        clock.unix_timestamp,
                        config.vesting_unlock_ts
                    );
                    return Err(ProgramError::Custom(3));
                }
                Ok(())
            }
        }
    }

    fn burn_nft(&self) -> ProgramResult {
        MplCoreProgram::burn(
            BurnMplCoreAssetAccounts {
                asset: self.accounts.nft_asset,
                collection: self.accounts.nft_collection,
                payer: self.accounts.payer,
                update_authority: self.accounts.nft_authority,
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            &[&[NftAuthorityV1::SEED, &[self.nft_authority_bump]]],
        )
    }

    fn refund_token(&self, config: &ConfigV1, balance: u64) -> ProgramResult {
        let signers_seeds: &[&[&[u8]]] = &[&[
            VaultV1::SEED,
            self.accounts.nft_asset.key.as_ref(),
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
            &[self.vault_bump],
        ]];

        TokenProgram::transfer_signed(
            TokenTransferAccounts {
                source: self.accounts.vault_ata,
                destination: self.accounts.payer_ata,
                authority: self.accounts.vault_pda,
                mint: self.accounts.token_mint,
                token_program: self.accounts.token_program,
            },
            TokenTransferArgs {
                signer_pubkeys: &[],
                amount: balance,
                decimals: config.mint_decimals,
            },
            signers_seeds,
        )
    }

    fn close_vault(&self) -> ProgramResult {
        let vault_seeds: &[&[u8]] = &[
            VaultV1::SEED,
            self.accounts.nft_asset.key.as_ref(),
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
            &[self.vault_bump],
        ];

        SystemProgram::close_ata(
            self.accounts.vault_ata,
            self.accounts.payer,
            self.accounts.vault_pda,
            self.accounts.token_program,
            vault_seeds,
        )?;

        SystemProgram::close_account_pda(self.accounts.vault_pda, self.accounts.payer)
    }
}

impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for BurnAndRefundV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        let accounts = BurnAndRefundV1Accounts::try_from(accounts)?;

        let (_, nft_authority_bump) =
            Pda::validate(accounts.nft_authority, &[NftAuthorityV1::SEED], program_id)?;

        Pda::validate(
            accounts.config_pda,
            &[
                ConfigV1::SEED,
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        let (_, vault_bump) = Pda::validate(
            accounts.vault_pda,
            &[
                VaultV1::SEED,
                accounts.nft_asset.key.as_ref(),
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        Ok(Self {
            accounts,
            nft_authority_bump,
            vault_bump,
        })
    }
}

impl<'a, 'info> ProcessInstruction for BurnAndRefundV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let config_data = self.accounts.config_pda.try_borrow_data()?;
        let config = ConfigV1::load(config_data.as_ref())?;

        let amount = {
            let vault_data = self.accounts.vault_pda.try_borrow_data()?;
            let vault = VaultV1::load(vault_data.as_ref())?;
            self.check_vesting(config, vault)?;
            vault.amount
        };

        self.burn_nft()?;
        self.refund_token(config, amount)?;
        self.close_vault()?;

        msg!("BurnAndRefund: burned NFT and refunded {} tokens", amount);

        Ok(())
    }
}

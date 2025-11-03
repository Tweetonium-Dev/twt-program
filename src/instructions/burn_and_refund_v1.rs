use mpl_core::instructions::BurnV1CpiBuilder;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::{Config, Vault},
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, ConfigAccount,
        MintAccount, MplCoreAccount, Pda, ProcessInstruction, SignerAccount, SystemProgram,
        TokenProgram, TransferArgs, VaultAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct BurnAndRefundV1Accounts<'a, 'info> {
    /// NFT owner — must sign to burn.
    /// Must be owner of `nft_token_account`.
    pub payer: &'a AccountInfo<'info>,

    /// NFT asset — must be burned.
    /// Must be valid MPL Core asset.
    pub nft_asset: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "vault"]` — escrow state.
    /// Must be readable.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Vault's ATA — source of refund token_mint.
    /// Must be writable, owned by `token_program`.
    pub vault_ata: &'a AccountInfo<'info>,

    /// User's ATA — receives refund.
    /// Must be writable, owned by `token_program`.
    pub payer_ata: &'a AccountInfo<'info>,

    /// PDA: `[program_id, token_mint, "config"]` — for price/refund logic.
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
        let [payer, nft_asset, vault_pda, vault_ata, payer_ata, config_pda, token_mint, token_program, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;

        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;
        WritableAccount::check(config_pda)?;

        AssociatedTokenAccount::check(vault_ata, vault_pda.key, token_mint.key, token_program.key)?;
        AssociatedTokenAccount::check(payer_ata, payer.key, token_mint.key, token_program.key)?;

        VaultAccount::check(vault_pda)?;
        ConfigAccount::check(config_pda)?;
        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreAccount::check(mpl_core)?;

        Ok(Self {
            payer,
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
    pub vault_bump: u8,
}

impl<'a, 'info> BurnAndRefundV1<'a, 'info> {
    fn check_vesting(&self, config: &Config, vault: &Vault) -> ProgramResult {
        let payer = self.accounts.payer;

        let clock = Clock::get()?;
        if clock.unix_timestamp < config.vesting_end_ts {
            msg!("Vesting not finished");
            return Err(ProgramError::Custom(4));
        }

        if vault.is_unlocked() {
            msg!("Vault already refunded");
            return Err(ProgramError::Custom(5));
        }

        if vault.owner != *payer.key {
            msg!("Vault owner mismatch");
            return Err(ProgramError::Custom(6));
        }

        Ok(())
    }

    fn burn_nft(&self) -> ProgramResult {
        let payer = self.accounts.payer;
        let nft_asset = self.accounts.nft_asset;
        let system_program = self.accounts.system_program;
        let mpl_core = self.accounts.mpl_core;

        msg!("Burn NFT");

        BurnV1CpiBuilder::new(mpl_core)
            .asset(nft_asset)
            .payer(payer)
            .authority(Some(payer))
            .system_program(Some(system_program))
            .invoke()?;

        msg!("Burn NFT successfully");

        Ok(())
    }

    fn refund_token(&self, config: &Config, balance: u64) -> ProgramResult {
        let payer = self.accounts.payer;
        let vault_pda = self.accounts.vault_pda;
        let vault_ata = self.accounts.vault_ata;
        let payer_ata = self.accounts.payer_ata;
        let token_mint = self.accounts.token_mint;
        let token_program = self.accounts.token_program;

        let signers_seeds: &[&[&[u8]]] = &[&[Vault::SEED, payer.key.as_ref(), &[self.vault_bump]]];

        TokenProgram::transfer_signed(
            TransferArgs {
                source: vault_ata,
                destination: payer_ata,
                authority: vault_pda,
                mint: token_mint,
                token_program,
                signer_pubkeys: &[],
                amount: balance,
                decimals: config.mint_decimals,
            },
            signers_seeds,
        )?;

        Ok(())
    }

    fn close_vault(&self) -> ProgramResult {
        let payer = self.accounts.payer;
        let vault_pda = self.accounts.vault_pda;
        let vault_ata = self.accounts.vault_ata;
        let token_program = self.accounts.token_program;

        let vault_seeds: &[&[u8]] = &[Vault::SEED, payer.key.as_ref(), &[self.vault_bump]];

        SystemProgram::close_ata(vault_ata, payer, vault_pda, token_program, vault_seeds)?;

        SystemProgram::close_account_pda(vault_pda, payer)?;

        Ok(())
    }
}

impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for BurnAndRefundV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        let accounts = BurnAndRefundV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.config_pda,
            &[Config::SEED, accounts.token_mint.key.as_ref()],
            program_id,
        )?;

        let (_, vault_bump) = Pda::validate(
            accounts.vault_pda,
            &[Vault::SEED, accounts.payer.key.as_ref()],
            program_id,
        )?;

        Ok(Self {
            accounts,
            vault_bump,
        })
    }
}

impl<'a, 'info> ProcessInstruction for BurnAndRefundV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let config_data = self.accounts.config_pda.data.borrow();
        let config = Config::load(&config_data)?;

        let amount = {
            let vault_data = self.accounts.vault_pda.data.borrow();
            let vault = Vault::load(&vault_data)?;

            let amount = vault.amount;
            // let balance = TokenProgram::get_balance(self.accounts.vault_ata, self.accounts.token_program)?;

            self.check_vesting(config, vault)?;

            amount
        };

        self.burn_nft()?;
        self.refund_token(config, amount)?;
        self.close_vault()?;

        msg!("BurnAndRefund: burned NFT and refunded {} tokens", amount);

        Ok(())
    }
}

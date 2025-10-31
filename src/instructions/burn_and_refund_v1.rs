use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::{Config, Vault, VAULT_SEED},
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, ConfigAccount, MintAccount, ProcessInstruction, SignerAccount, TokenProgram, TransferArgs, VaultAccount, WritableAccount
    },
};

#[derive(Debug)]
pub struct BurnAndRefundV1Accounts<'a, 'info> {
    pub authority: &'a AccountInfo<'info>,         // NFT owner
    pub nft_token_account: &'a AccountInfo<'info>, // Owner's NFT account
    pub nft_mint: &'a AccountInfo<'info>,          // NFT mint
    pub vault_pda: &'a AccountInfo<'info>,         // Vault PDA (escrow)
    pub vault_ata: &'a AccountInfo<'info>,         // Vault's ATA
    pub payer_ata: &'a AccountInfo<'info>,         // Owner's ATA
    pub vault_authority: &'a AccountInfo<'info>,   // Vault authority PDA
    pub config_pda: &'a AccountInfo<'info>,        // Config PDA
    pub token_asset: &'a AccountInfo<'info>,       // Token mint
    pub token_program: &'a AccountInfo<'info>,     // Token program
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for BurnAndRefundV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [
            authority,
            nft_token_account,
            nft_asset,
            vault_pda,
            vault_ata,
            payer_ata,
            vault_authority,
            config_pda,
            token_mint,
            token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;
        WritableAccount::check(nft_token_account)?;
        AssociatedTokenAccount::check(nft_token_account, authority.key, nft_asset.key, token_program.key)?;
        WritableAccount::check(vault_pda)?;
        VaultAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        AssociatedTokenAccount::check(vault_ata, vault_pda.key, token_mint.key, token_program.key)?;
        WritableAccount::check(payer_ata)?;
        AssociatedTokenAccount::check(payer_ata, authority.key, token_mint.key, token_program.key)?;
        WritableAccount::check(config_pda)?;
        ConfigAccount::check(config_pda)?;
        MintAccount::check(token_mint)?;

        Ok(Self {
            authority,
            nft_token_account,
            nft_mint: nft_asset,
            vault_pda,
            vault_ata,
            payer_ata,
            vault_authority,
            config_pda,
            token_asset: token_mint,
            token_program,
        })
    }
}

#[derive(Debug)]
pub struct BurnAndRefundV1<'a, 'info> {
    pub accounts: BurnAndRefundV1Accounts<'a, 'info>,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info> BurnAndRefundV1<'a, 'info> {
    fn check_vesting(&self, config: &Config, vault: &Vault) -> ProgramResult {
        let authority = self.accounts.authority;

        let clock = Clock::get()?;
        if clock.unix_timestamp < config.vesting_end_ts {
            msg!("Vesting no finished");
            return Err(ProgramError::Custom(4));
        }

        if vault.is_unlocked() {
            msg!("Vault already refunded");
            return Err(ProgramError::Custom(5));
        }

        if vault.owner != *authority.key {
            msg!("Vault owner mismatch");
            return Err(ProgramError::Custom(6));
        }

        Ok(())
    }

    fn burn_nft(&self, config: &Config) -> ProgramResult {
        let authority = self.accounts.authority;
        let nft_token_account = self.accounts.nft_token_account;
        let nft_mint = self.accounts.nft_mint;
        let token_program = self.accounts.token_program;

        TokenProgram::burn_nft(token_program, nft_token_account, nft_mint, authority, config.mint_decimals, &[])?;

        Ok(())
    }

    fn refund_token(&self, config: &Config, vault: &mut Vault) -> ProgramResult {
        let vault_ata = self.accounts.vault_ata;
        let payer_ata = self.accounts.payer_ata;
        let vault_authority = self.accounts.vault_authority;
        let config_pda = self.accounts.config_pda;
        let token_mint = self.accounts.token_asset;
        let token_program = self.accounts.token_program;

        let (expected_vault_auth, vault_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, config_pda.key.as_ref()], self.program_id);
        if expected_vault_auth != *vault_authority.key {
            msg!("Vault authority PDA mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        let signers_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, config_pda.key.as_ref(), &[vault_bump]]];

        TokenProgram::transfer_signed(
            TransferArgs {
                source: vault_ata,
                destination: payer_ata,
                authority: vault_authority,
                mint: token_mint,
                token_program,
                signer_pubkeys: &[],
                amount: vault.amount,
                decimals: config.mint_decimals,
            },
            signers_seeds,
        )?;

        vault.set_unlocked(true);

        Ok(())
    }
}

impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for BurnAndRefundV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        let accounts = BurnAndRefundV1Accounts::try_from(accounts)?;

        Ok(Self {
            accounts,
            program_id,
        })
    }
}

impl<'a, 'info> ProcessInstruction for BurnAndRefundV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let config_data = self.accounts.config_pda.data.borrow_mut();
        let config = Config::load(&config_data)?;

        let mut vault_data = self.accounts.vault_pda.data.borrow_mut();
        let mut vault = Vault::load_mut(&mut vault_data)?;

        self.check_vesting(&config, &vault)?;

        self.burn_nft(&config)?;

        self.refund_token(&config, &mut vault)?;

        let amount = vault.amount;

        msg!("BurnAndRefund: burned NFT and refunded {} tokens", amount);

        Ok(())
    }
}

use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::{Config, Vault},
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, ConfigAccount,
        MintAccount, Pda, ProcessInstruction, SignerAccount, TokenProgram, TransferArgs,
        VaultAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct BurnAndRefundV1Accounts<'a, 'info> {
    /// NFT owner — must sign to burn.
    /// Must be owner of `nft_token_account`.
    pub authority: &'a AccountInfo<'info>,

    /// User's NFT token account — holds 1 NFT.
    /// Must be writable, amount = 1, owned by `token_program`.
    pub nft_token_account: &'a AccountInfo<'info>,

    /// NFT asset — must be burned.
    /// Must be valid MPL Core asset.
    pub nft_asset: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "vault_authority"]`.
    /// Signs CPI to transfer from `vault_ata`.
    /// Must be PDA, not required to sign.
    pub vault_authority: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "vault"]` — escrow state.
    /// Must be readable.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Vault's ATA — source of refund token_mint.
    /// Must be writable, owned by `token_program`.
    pub vault_ata: &'a AccountInfo<'info>,

    /// User's ATA — receives refund.
    /// Must be writable, owned by `token_program`.
    pub payer_ata: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "config"]` — for price/refund logic.
    /// Must be readable.
    pub config_pda: &'a AccountInfo<'info>,

    /// Token mint — must match config (e.g. ZDLT.
    /// Must be valid mint.
    pub token_mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_asset.owner`.
    pub token_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for BurnAndRefundV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [
            authority,
            nft_token_account,
            nft_asset,
            vault_authority,
            vault_pda,
            vault_ata,
            payer_ata,
            config_pda,
            token_mint,
            token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;

        WritableAccount::check(nft_token_account)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;
        WritableAccount::check(config_pda)?;

        AssociatedTokenAccount::check(
            nft_token_account,
            authority.key,
            nft_asset.key,
            token_program.key,
        )?;
        AssociatedTokenAccount::check(vault_ata, vault_pda.key, token_mint.key, token_program.key)?;
        AssociatedTokenAccount::check(payer_ata, authority.key, token_mint.key, token_program.key)?;

        VaultAccount::check(vault_pda)?;
        ConfigAccount::check(config_pda)?;
        MintAccount::check(token_mint)?;

        Ok(Self {
            authority,
            nft_token_account,
            nft_asset,
            vault_authority,
            vault_pda,
            vault_ata,
            payer_ata,
            config_pda,
            token_mint,
            token_program,
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
        let authority = self.accounts.authority;

        let clock = Clock::get()?;
        if clock.unix_timestamp < config.vesting_end_ts {
            msg!("Vesting not finished");
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
        let nft_asset = self.accounts.nft_asset;
        let token_program = self.accounts.token_program;

        TokenProgram::burn_nft(
            token_program,
            nft_token_account,
            nft_asset,
            authority,
            config.mint_decimals,
            &[],
        )?;

        Ok(())
    }

    fn refund_token(&self, config: &Config, vault: &mut Vault) -> ProgramResult {
        let authority = self.accounts.authority;
        let vault_ata = self.accounts.vault_ata;
        let payer_ata = self.accounts.payer_ata;
        let vault_authority = self.accounts.vault_authority;
        let token_mint = self.accounts.token_mint;
        let token_program = self.accounts.token_program;

        let signers_seeds: &[&[&[u8]]] =
            &[&[Vault::SEED, authority.key.as_ref(), &[self.vault_bump]]];

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

        Pda::validate(accounts.config_pda, &[Config::SEED], program_id)?;

        let (_, bump) = Pda::validate(
            accounts.vault_pda,
            &[Vault::SEED, accounts.authority.key.as_ref()],
            program_id,
        )?;

        Pda::validate(accounts.config_pda, &[Config::SEED], program_id)?;

        Ok(Self {
            accounts,
            vault_bump: bump,
        })
    }
}

impl<'a, 'info> ProcessInstruction for BurnAndRefundV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let config_data = self.accounts.config_pda.data.borrow_mut();
        let config = Config::load(&config_data)?;

        let mut vault_data = self.accounts.vault_pda.data.borrow_mut();
        let vault = Vault::load_mut(&mut vault_data)?;

        self.check_vesting(config, vault)?;

        self.burn_nft(config)?;

        self.refund_token(config, vault)?;

        let amount = vault.amount;

        msg!("BurnAndRefund: burned NFT and refunded {} tokens", amount);

        Ok(())
    }
}

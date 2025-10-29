use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token_interface::instruction as token_instruction;

use crate::{
    states::{Config, Vault, VAULT_SEED},
    utils::{AccountCheck, ProcessInstruction, SignerAccount, WritableAccount},
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
    pub token_program: &'a AccountInfo<'info>,     // Token program
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for BurnAndRefundV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, nft_token_account, nft_mint, vault_pda, vault_ata, payer_ata, vault_authority, config_pda, token_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;
        WritableAccount::check(nft_token_account)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;
        WritableAccount::check(config_pda)?;

        Ok(Self {
            authority,
            nft_token_account,
            nft_mint,
            vault_pda,
            vault_ata,
            payer_ata,
            vault_authority,
            config_pda,
            token_program,
        })
    }
}

#[derive(Debug)]
pub struct BurnAndRefundV1<'a, 'info> {
    pub accounts: BurnAndRefundV1Accounts<'a, 'info>,
    pub program_id: &'a Pubkey,
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
        let authority = self.accounts.authority;
        let nft_token_account = self.accounts.nft_token_account;
        let nft_mint = self.accounts.nft_mint;
        let vault_pda = self.accounts.vault_pda;
        let vault_ata = self.accounts.vault_ata;
        let payer_ata = self.accounts.payer_ata;
        let vault_authority = self.accounts.vault_authority;
        let config_pda = self.accounts.config_pda;
        let token_program = self.accounts.token_program;

        // Load config and vault
        let cfg = Config::load(&config_pda.data.borrow())?;
        let mut vault = Vault::load(&vault_pda.data.borrow())?;

        // Vesting check
        let clock = Clock::get()?;
        if clock.unix_timestamp < cfg.vesting_end_ts {
            msg!("Vesting no finished");
            return Err(ProgramError::Custom(4));
        }

        if vault.is_unlocked {
            msg!("Vault already refunded");
            return Err(ProgramError::Custom(5));
        }

        if vault.owner != *authority.key {
            msg!("Vault owner mismatch");
            return Err(ProgramError::Custom(6));
        }

        // Burn NFT (authority signs)
        let burn_ix = token_instruction::burn(
            token_program.key,
            nft_token_account.key,
            nft_mint.key,
            authority.key,
            &[],
            1,
        )?;

        invoke(
            &burn_ix,
            &[
                nft_token_account.clone(),
                nft_mint.clone(),
                authority.clone(),
                token_program.clone(),
            ],
        )?;

        // Refund from vault to payer
        let (expected_vault_auth, vault_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, config_pda.key.as_ref()], self.program_id);
        if expected_vault_auth != *vault_authority.key {
            msg!("Vault authority PDA mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        let refund_ix = token_instruction::transfer(
            token_program.key,
            vault_ata.key,
            payer_ata.key,
            vault_authority.key,
            &[],
            vault.amount,
        )?;

        invoke_signed(
            &refund_ix,
            &[
                vault_ata.clone(),
                payer_ata.clone(),
                vault_authority.clone(),
                token_program.clone(),
            ],
            &[&[VAULT_SEED, config_pda.key.as_ref(), &[vault_bump]]],
        )?;

        vault.is_unlocked = true;
        vault.serialize(&mut &mut vault_pda.data.borrow_mut()[..])?;

        msg!(
            "BurnAndRefund: burned NFT and refunded {} tokens",
            vault.amount
        );

        Ok(())
    }
}

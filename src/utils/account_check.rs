use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{Config, Vault},
    utils::{
        AssociatedTokenProgram, MINT_2022_MIN_LEN, MINT_LEN, TOKEN_2022_PROGRAM_ID,
        TOKEN_ACCOUNT_2022_MIN_LEN, TOKEN_ACCOUNT_LEN, TOKEN_PROGRAM_ID,
    },
};

pub trait AccountCheck {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult;
}

pub trait AssociatedTokenAccountCheck {
    fn check<'info>(
        account: &AccountInfo<'info>,
        wallet: &Pubkey,
        mint: &Pubkey,
        token_program_id: &Pubkey,
    ) -> ProgramResult;
}

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.is_signer {
            msg!("SignerAccount: account {} must be a signer", account.key);
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }
}

pub struct UninitializedAccount;

impl AccountCheck for UninitializedAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.lamports() == 0 || !account.data_is_empty() {
            msg!(
                "UninitializedAccount: account {} is already initialized",
                account.key
            );
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}

pub struct WritableAccount;

impl AccountCheck for WritableAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.is_writable {
            msg!("WritableAccount: account {} must be writable", account.key);
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

pub struct MintAccount;

impl AccountCheck for MintAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        let owner = account.owner;

        if *owner == TOKEN_2022_PROGRAM_ID {
            if account.data_len() > MINT_2022_MIN_LEN {
                msg!(
                    "MintAccount: invalid Token-2022 mint length (expected ≤ {}, found {}) for account {}",
                    MINT_2022_MIN_LEN,
                    account.data_len(),
                    account.key
                );
                return Err(ProgramError::InvalidAccountData);
            }
            return Ok(());
        }

        if *owner == TOKEN_PROGRAM_ID {
            if account.data_len() != MINT_LEN {
                msg!(
                    "MintAccount: invalid Token mint length (expected {}, found {}) for account {}",
                    MINT_LEN,
                    account.data_len(),
                    account.key
                );
                return Err(ProgramError::InvalidAccountData);
            }
            return Ok(());
        }

        msg!(
            "MintAccount: invalid mint owner {} (expected SPL Token or Token-2022 program)",
            owner
        );
        Err(ProgramError::InvalidAccountOwner)
    }
}

pub struct TokenAccount;

impl AccountCheck for TokenAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        let owner = account.owner;

        if *owner == TOKEN_2022_PROGRAM_ID {
            if account.data_len() < TOKEN_ACCOUNT_2022_MIN_LEN {
                msg!(
                    "TokenAccount: invalid Token-2022 account length (expected ≥ {}, found {}) for account {}",
                    TOKEN_ACCOUNT_2022_MIN_LEN,
                    account.data_len(),
                    account.key
                );
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        if *owner == TOKEN_PROGRAM_ID {
            if account.data_len() != TOKEN_ACCOUNT_LEN {
                msg!(
                    "TokenAccount: invalid SPL Token account length (expected {}, found {}) for account {}",
                    TOKEN_ACCOUNT_LEN,
                    account.data_len(),
                    account.key
                );
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        msg!(
            "TokenAccount: invalid owner {} for account {} (expected SPL Token or Token-2022 program)",
            owner,
            account.key
        );
        Err(ProgramError::InvalidAccountOwner)
    }
}

pub struct ConfigAccount;

impl AccountCheck for ConfigAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.owner != &crate::ID {
            msg!(
                "ConfigAccount: invalid owner {} (expected program {})",
                account.owner,
                crate::ID
            );
            return Err(ProgramError::InvalidAccountOwner);
        }

        if account.data_len() != Config::LEN {
            msg!(
                "ConfigAccount: invalid data length (expected {}, found {}) for account {}",
                Config::LEN,
                account.data_len(),
                account.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

pub struct VaultAccount;

impl AccountCheck for VaultAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.owner != &crate::ID {
            msg!(
                "VaultAccount: invalid owner {} (expected program {})",
                account.owner,
                crate::ID
            );
            return Err(ProgramError::InvalidAccountOwner);
        }

        if account.data_len() != Vault::LEN {
            msg!(
                "VaultAccount: invalid data length (expected {}, found {}) for account {}",
                Vault::LEN,
                account.data_len(),
                account.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

pub struct AssociatedTokenAccount;

impl AssociatedTokenAccountCheck for AssociatedTokenAccount {
    fn check<'info>(
        account: &AccountInfo<'info>,
        wallet: &Pubkey,
        mint: &Pubkey,
        token_program_id: &Pubkey,
    ) -> ProgramResult {
        TokenAccount::check(account)?;
        AssociatedTokenProgram::check(account, wallet, mint, token_program_id)?;
        Ok(())
    }
}

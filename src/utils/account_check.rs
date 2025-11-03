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

pub trait AccountUninitializedCheck {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult;
}

pub trait AssociatedTokenAccountCheck {
    fn check<'info>(
        account: &AccountInfo<'info>,
        wallet: &Pubkey,
        mint: &Pubkey,
        token_program_id: &Pubkey,
    ) -> ProgramResult;
}

pub struct WritableAccount;

impl AccountCheck for WritableAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.is_writable {
            msg!("Account need writable: {}", account.key);
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl AccountUninitializedCheck for WritableAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
            msg!("Account are initialized: {}", account.key);
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.is_signer {
            msg!("Account need signer: {}", account.key);
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }
}

pub struct MplCoreAccount;

impl AccountCheck for MplCoreAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if *account.key != mpl_core::ID {
            msg!("Mpl core invalid");
            return Err(ProgramError::IncorrectProgramId);
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
                    "Mint 2022 account length should be {}, but found {}",
                    account.data_len(),
                    MINT_2022_MIN_LEN
                );
                return Err(ProgramError::InvalidAccountData);
            }
            return Ok(());
        }

        if *owner == TOKEN_PROGRAM_ID {
            if account.data_len() != MINT_LEN {
                msg!(
                    "Mint account length should be {}, but found {}",
                    account.data_len(),
                    MINT_LEN
                );
                return Err(ProgramError::InvalidAccountData);
            }
            return Ok(());
        }

        msg!("Mint account invalid owner: {}", account.key);
        Err(ProgramError::InvalidAccountOwner)
    }
}

impl AccountUninitializedCheck for MintAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
            msg!("Account should be uninitalized: {}", account.key);
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}

pub struct TokenAccount;

impl AccountCheck for TokenAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        let owner = account.owner;

        if *owner == TOKEN_2022_PROGRAM_ID {
            if account.data_len() < TOKEN_ACCOUNT_2022_MIN_LEN {
                msg!(
                    "Token 2022 account length should be {}, but found {}",
                    account.data_len(),
                    TOKEN_ACCOUNT_2022_MIN_LEN
                );
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        if *owner == TOKEN_PROGRAM_ID {
            if account.data_len() != TOKEN_ACCOUNT_LEN {
                msg!(
                    "Token account length should be {}, but found {}",
                    account.data_len(),
                    TOKEN_ACCOUNT_LEN
                );
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        msg!("Invalid Token account {}", account.key);
        Err(ProgramError::InvalidAccountOwner)
    }
}

pub struct ConfigAccount;

impl AccountCheck for ConfigAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.owner != &crate::ID {
            msg!("Config should be owned by program");
            return Err(ProgramError::InvalidAccountOwner);
        }

        if account.data_len() != Config::LEN {
            msg!(
                "Config length should be {}, but found {}",
                account.data_len(),
                Config::LEN
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
            msg!("Vault should be owned by program");
            return Err(ProgramError::InvalidAccountOwner);
        }

        if account.data_len() != Vault::LEN {
            msg!(
                "Vault length should be {}, but found {}",
                account.data_len(),
                Config::LEN
            );
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl AccountUninitializedCheck for VaultAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
            msg!("Vault should be uninitalized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}

pub struct MintedUserAccount;

impl AccountUninitializedCheck for MintedUserAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
            msg!("User already mint NFT");
            return Err(ProgramError::AccountAlreadyInitialized);
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

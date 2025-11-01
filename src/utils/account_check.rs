use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_sdk_ids::system_program;

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
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl AccountUninitializedCheck for WritableAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        let owner = Pubkey::new_from_array(account.owner.to_bytes());
        let system_program = Pubkey::new_from_array(system_program::ID.to_bytes());

        if owner != system_program {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

pub struct MplCoreAccount;

impl AccountCheck for MplCoreAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if *account.owner != mpl_core::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

pub struct MplCoreAsset;

impl AccountCheck for MplCoreAsset {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if *account.owner != mpl_core::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

impl AccountUninitializedCheck for MplCoreAsset {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}

pub struct MintAccount;

impl AccountCheck for MintAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        let owner = account.owner;

        if *owner == TOKEN_2022_PROGRAM_ID {
            if account.data_len() < MINT_2022_MIN_LEN {
                return Err(ProgramError::InvalidAccountData);
            }
            return Ok(());
        }

        if *owner == TOKEN_PROGRAM_ID {
            if account.data_len() != MINT_LEN {
                return Err(ProgramError::InvalidAccountData);
            }
            return Ok(());
        }

        Err(ProgramError::InvalidAccountOwner)
    }
}

impl AccountUninitializedCheck for MintAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
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
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        if *owner == TOKEN_PROGRAM_ID {
            if account.data_len() != TOKEN_ACCOUNT_LEN {
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        Err(ProgramError::InvalidAccountOwner)
    }
}

pub struct ConfigAccount;

impl AccountCheck for ConfigAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.owner != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if account.data_len() != Config::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

pub struct VaultAccount;

impl AccountCheck for VaultAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.owner != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if account.data_len() != Vault::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl AccountUninitializedCheck for VaultAccount {
    fn check_uninitialized<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if !account.data_is_empty() {
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

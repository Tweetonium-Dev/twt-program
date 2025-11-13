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
        if account.lamports() != 0 || !account.data_is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ASSOCIATED_TOKEN_PROGRAM_ID;

    // --- Test Helpers ---

    const PROGRAM_ID: Pubkey = crate::ID;

    const WRONG_PROGRAM_ID: Pubkey = Pubkey::new_from_array([2u8; 32]);

    fn mock_account_info(
        is_signer: bool,
        is_writable: bool,
        owner: Pubkey,
        data_len: usize,
    ) -> AccountInfo<'static> {
        crate::utils::mock::mock_account(
            Pubkey::new_unique(),
            is_signer,
            is_writable,
            1,
            data_len,
            owner,
        )
    }

    fn mock_account_info_from_key(
        key: Pubkey,
        is_signer: bool,
        is_writable: bool,
        owner: Pubkey,
        data_len: usize,
    ) -> AccountInfo<'static> {
        crate::utils::mock::mock_account(key, is_signer, is_writable, 1, data_len, owner)
    }

    fn mock_uninitialized_account_info() -> AccountInfo<'static> {
        crate::utils::mock::mock_account(
            Pubkey::new_unique(),
            false,
            true,
            0,
            0,
            Pubkey::new_unique(),
        )
    }

    // --- Test Cases ---

    #[test]
    fn test_signer_account() {
        let acc = mock_account_info(true, false, Pubkey::new_unique(), 0);
        assert!(SignerAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, Pubkey::new_unique(), 0);
        assert_eq!(
            SignerAccount::check(&acc).unwrap_err(),
            ProgramError::MissingRequiredSignature
        );
    }

    #[test]
    fn test_uninitialized_account() {
        let acc = mock_uninitialized_account_info();
        assert!(UninitializedAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, Pubkey::new_unique(), 10);
        assert_eq!(
            UninitializedAccount::check(&acc).unwrap_err(),
            ProgramError::AccountAlreadyInitialized
        );
    }

    #[test]
    fn test_writable_account() {
        let acc = mock_account_info(false, true, Pubkey::new_unique(), 0);
        assert!(WritableAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, Pubkey::new_unique(), 10);
        assert_eq!(
            WritableAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );
    }

    #[test]
    fn test_mint_account_with_token_program() {
        let acc = mock_account_info(false, false, TOKEN_PROGRAM_ID, MINT_LEN);
        assert!(MintAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, TOKEN_PROGRAM_ID, MINT_LEN + 1);
        assert_eq!(
            MintAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );

        let acc = mock_account_info(false, false, TOKEN_2022_PROGRAM_ID, MINT_2022_MIN_LEN);
        assert!(MintAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, TOKEN_2022_PROGRAM_ID, MINT_2022_MIN_LEN + 1);
        assert_eq!(
            MintAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );

        let acc = mock_account_info(false, false, Pubkey::new_unique(), MINT_LEN);
        assert_eq!(
            MintAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountOwner
        );
    }

    #[test]
    fn test_token_account_check() {
        let acc = mock_account_info(false, false, TOKEN_PROGRAM_ID, TOKEN_ACCOUNT_LEN);
        assert!(TokenAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, TOKEN_PROGRAM_ID, TOKEN_ACCOUNT_LEN + 1);
        assert_eq!(
            TokenAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );

        let acc = mock_account_info(
            false,
            false,
            TOKEN_2022_PROGRAM_ID,
            TOKEN_ACCOUNT_2022_MIN_LEN,
        );
        assert!(TokenAccount::check(&acc).is_ok());

        let acc = mock_account_info(
            false,
            false,
            TOKEN_PROGRAM_ID,
            TOKEN_ACCOUNT_2022_MIN_LEN + 1,
        );
        assert_eq!(
            TokenAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );

        let acc = mock_account_info(false, false, Pubkey::new_unique(), TOKEN_ACCOUNT_LEN);
        assert_eq!(
            TokenAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountOwner
        );
    }

    #[test]
    fn test_config_account() {
        let acc = mock_account_info(false, false, PROGRAM_ID, Config::LEN);
        assert!(ConfigAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, PROGRAM_ID, Config::LEN + 1);
        assert_eq!(
            ConfigAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );

        let acc = mock_account_info(false, false, WRONG_PROGRAM_ID, Config::LEN);
        assert_eq!(
            ConfigAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountOwner
        );
    }

    #[test]
    fn test_vault_account() {
        let acc = mock_account_info(false, false, PROGRAM_ID, Vault::LEN);
        assert!(VaultAccount::check(&acc).is_ok());

        let acc = mock_account_info(false, false, PROGRAM_ID, Vault::LEN + 1);
        assert_eq!(
            VaultAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountData
        );

        let acc = mock_account_info(false, false, WRONG_PROGRAM_ID, Vault::LEN);
        assert_eq!(
            VaultAccount::check(&acc).unwrap_err(),
            ProgramError::InvalidAccountOwner
        );
    }

    #[test]
    fn test_associated_token_account() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let token_program_id = TOKEN_PROGRAM_ID;

        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        let acc = mock_account_info_from_key(
            expected_ata,
            false,
            true,
            token_program_id,
            TOKEN_ACCOUNT_LEN,
        );

        assert!(AssociatedTokenAccount::check(&acc, &wallet, &mint, &token_program_id).is_ok());
    }
}

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
};
use solana_sdk_ids::system_program;

pub trait AccountCheck {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult;
}

pub trait OptionalAccountCheck {
    fn check_optional<'info>(account: Option<&AccountInfo<'info>>) -> ProgramResult;
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

impl OptionalAccountCheck for WritableAccount {
    fn check_optional<'info>(account: Option<&AccountInfo<'info>>) -> ProgramResult {
        if let Some(account) = account {
            if !account.is_writable {
                return Err(ProgramError::InvalidAccountData);
            }
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

impl OptionalAccountCheck for SignerAccount {
    fn check_optional<'info>(account: Option<&AccountInfo<'info>>) -> ProgramResult {
        if let Some(account) = account {
            if !account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
        }

        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.owner != &system_program::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

impl OptionalAccountCheck for SystemAccount {
    fn check_optional<'info>(account: Option<&AccountInfo<'info>>) -> ProgramResult {
        if let Some(account) = account {
            if account.owner != &system_program::ID {
                return Err(ProgramError::InvalidAccountOwner);
            }
        }

        Ok(())
    }
}

pub struct MplCoreAccount;

impl AccountCheck for MplCoreAccount {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        let account = Pubkey::new_from_array(account.owner.to_bytes());
        let mpl_core = Pubkey::new_from_array(mpl_core::ID.to_bytes());
        if account != mpl_core {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::*;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_writable_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            true,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(WritableAccount::check(&acc).is_ok());
    }

    #[test]
    fn test_writable_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(WritableAccount::check(&acc).is_err());
    }

    #[test]
    fn test_optional_writable_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            true,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(WritableAccount::check_optional(Some(&acc)).is_ok());
    }

    #[test]
    fn test_optional_writable_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(WritableAccount::check_optional(Some(&acc)).is_err());
    }

    #[test]
    fn test_optional_writable_account_check_none() {
        assert!(WritableAccount::check_optional(None).is_ok());
    }

    #[test]
    fn test_signer_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            true,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(SignerAccount::check(&acc).is_ok());
    }

    #[test]
    fn test_signer_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(SignerAccount::check(&acc).is_err());
    }

    #[test]
    fn test_optional_signer_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            true,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(SignerAccount::check_optional(Some(&acc)).is_ok());
    }

    #[test]
    fn test_optional_signer_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(SignerAccount::check_optional(Some(&acc)).is_err());
    }

    #[test]
    fn test_optional_signer_account_check_none() {
        assert!(SignerAccount::check_optional(None).is_ok());
    }

    #[test]
    fn test_system_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            system_program::ID,
        );
        assert!(SystemAccount::check(&acc).is_ok());
    }

    #[test]
    fn test_system_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(SystemAccount::check(&acc).is_err());
    }

    #[test]
    fn test_optional_system_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            system_program::ID,
        );
        assert!(SystemAccount::check_optional(Some(&acc)).is_ok());
    }

    #[test]
    fn test_optional_system_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(SystemAccount::check_optional(Some(&acc)).is_err());
    }

    #[test]
    fn test_optional_system_account_check_none() {
        assert!(SystemAccount::check_optional(None).is_ok());
    }

    #[test]
    fn test_mpl_core_account_check_success() {
        let acc = new_test_account(
            Pubkey::new_unique(), 
            false, 
            false, 
            10, 
            0, 
            Pubkey::new_from_array(mpl_core::ID.to_bytes())
        );
        assert!(MplCoreAccount::check(&acc).is_ok());
    }

    #[test]
    fn test_mpl_core_account_check_failed() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10,
            0,
            Pubkey::new_unique(),
        );
        assert!(MplCoreAccount::check(&acc).is_err());
    }
}

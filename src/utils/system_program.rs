use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    system_instruction, system_program,
};

use crate::utils::{AccountCheck, TokenProgram};

pub struct SystemProgram;

impl SystemProgram {
    pub fn transfer<'info>(
        source: &AccountInfo<'info>,
        destination: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        amount: u64,
    ) -> ProgramResult {
        let ix = system_instruction::transfer(source.key, destination.key, amount);
        invoke(
            &ix,
            &[source.clone(), destination.clone(), system_program.clone()],
        )
    }

    pub fn close_account_pda<'info>(
        account: &AccountInfo<'info>,
        destination: &AccountInfo<'info>,
    ) -> ProgramResult {
        let lamports = account.lamports();
        if lamports == 0 {
            return Ok(());
        }

        **account.lamports.borrow_mut() = 0;
        **destination.lamports.borrow_mut() += lamports;

        let mut data = account.try_borrow_mut_data()?;
        data.fill(0);

        Ok(())
    }

    pub fn close_ata<'info>(
        ata: &AccountInfo<'info>,
        destination: &AccountInfo<'info>,
        owner_pda: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        seeds: &[&[u8]],
    ) -> ProgramResult {
        let balance = TokenProgram::get_balance(ata, token_program)?;

        if balance != 0 {
            msg!("ATA still holds {} tokens", balance);
            return Err(ProgramError::InvalidAccountData);
        }

        let data = vec![9u8];

        let accounts = vec![
            AccountMeta::new(*ata.key, false),
            AccountMeta::new(*destination.key, false),
            AccountMeta::new_readonly(*owner_pda.key, true),
        ];

        let ix = Instruction {
            program_id: *token_program.key,
            accounts,
            data,
        };

        let signer_seeds: &[&[&[u8]]] = &[seeds];

        invoke_signed(
            &ix,
            &[
                ata.clone(),
                destination.clone(),
                owner_pda.clone(),
                token_program.clone(),
            ],
            signer_seeds,
        )
    }
}

impl AccountCheck for SystemProgram {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if account.key != &system_program::ID {
            msg!("Account should be system program {}", account.key);
            return Err(ProgramError::IncorrectProgramId);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;

    // --- Test Helpers ---

    fn mock_account_info(key: Pubkey, lamports: u64, data_len: usize) -> AccountInfo<'static> {
        crate::utils::mock::mock_account(key, false, true, lamports, data_len, Pubkey::new_unique())
    }

    // --- Test Cases ---

    #[test]
    fn test_close_account_pda_transfers_lamports_and_clears_data() {
        let key = Pubkey::new_unique();
        let dest_key = Pubkey::new_unique();

        let account = mock_account_info(key, 10_000, 16);
        let destination = mock_account_info(dest_key, 5_000, 0);

        let result = SystemProgram::close_account_pda(&account, &destination);

        assert!(result.is_ok());
        assert_eq!(
            **account.lamports.borrow(),
            0,
            "account lamports should now be 0"
        );
        assert_eq!(
            **destination.lamports.borrow(),
            15_000,
            "destination should receive 10_000 lamports more"
        );
        assert!(
            account.data.borrow().iter().all(|b| *b == 0),
            "data should be zeroed"
        );
    }

    #[test]
    fn test_close_account_pda_noop_when_zero_balance() {
        let key = Pubkey::new_unique();
        let dest_key = Pubkey::new_unique();

        let account = mock_account_info(key, 0, 8);
        let destination = mock_account_info(dest_key, 0, 0);

        let result = SystemProgram::close_account_pda(&account, &destination);

        assert!(result.is_ok());
        assert_eq!(**account.lamports.borrow(), 0);
        assert_eq!(**destination.lamports.borrow(), 0);
    }

    #[test]
    fn test_check_valid_system_program() {
        let account = mock_account_info(system_program::ID, 0, 0);
        assert!(SystemProgram::check(&account).is_ok());
    }

    #[test]
    fn test_check_invalid_system_program() {
        let account = mock_account_info(Pubkey::new_unique(), 0, 0);
        let result = SystemProgram::check(&account);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(ProgramError::IncorrectProgramId));
    }
}

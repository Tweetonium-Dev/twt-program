use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
};
use spl_token::instruction::initialize_account3;

use crate::utils::{
    AccountCheck, Pda, TokenProgram, UninitializedAccount, TOKEN_2022_PROGRAM_ID,
    TOKEN_ACCOUNT_2022_MIN_LEN, TOKEN_ACCOUNT_LEN,
};

pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub struct AssociatedTokenProgram;

impl AssociatedTokenProgram {
    pub fn init<'a, 'info>(
        accounts: InitAssociatedTokenProgramAccounts<'a, 'info>,
    ) -> ProgramResult {
        let seeds = &[
            accounts.wallet.key.as_ref(),
            accounts.token_program.key.as_ref(),
            accounts.mint.key.as_ref(),
        ];

        Pda::validate(accounts.ata, seeds, &ASSOCIATED_TOKEN_PROGRAM_ID)?;

        let ix = match TokenProgram::detect_token_program(accounts.token_program)? {
            TokenProgram::Token => initialize_account3(
                accounts.token_program.key,
                accounts.ata.key,
                accounts.mint.key,
                accounts.wallet.key,
            )?,
            TokenProgram::Token2022 => Instruction {
                program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
                accounts: vec![
                    AccountMeta::new(*accounts.payer.key, accounts.payer.is_signer),
                    AccountMeta::new(*accounts.ata.key, false),
                    AccountMeta::new_readonly(*accounts.wallet.key, false),
                    AccountMeta::new_readonly(*accounts.mint.key, false),
                    AccountMeta::new_readonly(*accounts.system_program.key, false),
                    AccountMeta::new_readonly(*accounts.token_program.key, false),
                ],
                data: vec![0],
            },
        };

        invoke(
            &ix,
            &[
                accounts.payer.clone(),
                accounts.wallet.clone(),
                accounts.ata.clone(),
                accounts.mint.clone(),
                accounts.system_program.clone(),
                accounts.token_program.clone(),
                accounts.associated_token_program.clone(),
            ],
        )
    }

    pub fn init_if_needed<'a, 'info>(
        accounts: InitAssociatedTokenProgramAccounts<'a, 'info>,
    ) -> ProgramResult {
        if UninitializedAccount::check(accounts.ata).is_ok() {
            Self::init(accounts)?;
        }

        Ok(())
    }

    pub fn check<'info>(
        ata: &AccountInfo<'info>,
        wallet: &Pubkey,
        mint: &Pubkey,
        token_program_id: &Pubkey,
    ) -> ProgramResult {
        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        if ata.key != &expected_ata {
            msg!(
                "Invalid ATA seeds. Actual {}, Expected {}, Wallet {}",
                ata.key,
                &expected_ata,
                wallet
            );
            return Err(ProgramError::InvalidSeeds);
        }

        if ata.owner != token_program_id {
            msg!("Invalid ATA account {}", ata.key);
            return Err(ProgramError::InvalidAccountOwner);
        }

        let expected_len = if token_program_id == &TOKEN_2022_PROGRAM_ID {
            TOKEN_ACCOUNT_2022_MIN_LEN..=usize::MAX
        } else {
            TOKEN_ACCOUNT_LEN..=TOKEN_ACCOUNT_LEN
        };

        if !expected_len.contains(&ata.data_len()) {
            msg!("Invalid ATA data {}", ata.key);
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

pub struct InitAssociatedTokenProgramAccounts<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub wallet: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub associated_token_program: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
    pub ata: &'a AccountInfo<'info>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{mock::mock_account, TOKEN_PROGRAM_ID};

    #[test]
    fn test_init_if_needed_skips_initialized() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let token_program_id = TOKEN_PROGRAM_ID;

        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        let ata = mock_account(
            expected_ata,
            false,
            true,
            1,
            TOKEN_ACCOUNT_LEN - 5,
            token_program_id,
        );

        let payer = mock_account(Pubkey::new_unique(), true, true, 1, 0, Pubkey::new_unique());
        let wallet = mock_account(
            Pubkey::new_unique(),
            false,
            false,
            1,
            0,
            Pubkey::new_unique(),
        );
        let mint = mock_account(
            Pubkey::new_unique(),
            false,
            false,
            1,
            0,
            Pubkey::new_unique(),
        );
        let token_program =
            mock_account(TOKEN_PROGRAM_ID, false, false, 1, 0, Pubkey::new_unique());
        let system_program =
            mock_account(Pubkey::default(), false, false, 1, 0, Pubkey::new_unique());
        let associated_token_program = mock_account(
            ASSOCIATED_TOKEN_PROGRAM_ID,
            false,
            false,
            1,
            0,
            Pubkey::new_unique(),
        );

        let accounts = InitAssociatedTokenProgramAccounts {
            payer: &payer,
            wallet: &wallet,
            mint: &mint,
            token_program: &token_program,
            associated_token_program: &associated_token_program,
            system_program: &system_program,
            ata: &ata,
        };

        assert!(AssociatedTokenProgram::init_if_needed(accounts).is_ok());
    }

    #[test]
    fn test_check_valid_ata() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let token_program_id = TOKEN_PROGRAM_ID;

        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        let ata_acc = mock_account(
            expected_ata,
            false,
            true,
            1,
            TOKEN_ACCOUNT_LEN,
            token_program_id,
        );

        assert!(AssociatedTokenProgram::check(&ata_acc, &wallet, &mint, &token_program_id).is_ok());
    }

    #[test]
    fn test_check_invalid_seeds() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let token_program_id = TOKEN_PROGRAM_ID;
        let wrong_key = Pubkey::new_unique();

        let ata_acc = mock_account(
            wrong_key,
            false,
            true,
            1,
            TOKEN_ACCOUNT_LEN,
            token_program_id,
        );

        assert_eq!(
            AssociatedTokenProgram::check(&ata_acc, &wallet, &mint, &token_program_id).unwrap_err(),
            ProgramError::InvalidSeeds,
        );
    }

    #[test]
    fn test_check_invalid_owner() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let token_program_id = TOKEN_PROGRAM_ID;

        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        let wrong_owner = Pubkey::new_unique();

        let ata_acc = mock_account(expected_ata, false, true, 1, TOKEN_ACCOUNT_LEN, wrong_owner);

        assert_eq!(
            AssociatedTokenProgram::check(&ata_acc, &wallet, &mint, &token_program_id).unwrap_err(),
            ProgramError::InvalidAccountOwner,
        );
    }

    #[test]
    fn test_check_invalid_data_length() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let token_program_id = TOKEN_PROGRAM_ID;

        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        let ata_acc = mock_account(
            expected_ata,
            false,
            true,
            1,
            TOKEN_ACCOUNT_LEN - 5,
            token_program_id,
        );

        assert_eq!(
            AssociatedTokenProgram::check(&ata_acc, &wallet, &mint, &token_program_id).unwrap_err(),
            ProgramError::InvalidAccountData,
        );
    }
}

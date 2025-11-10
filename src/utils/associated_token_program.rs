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
        args: InitAssociatedTokenProgramArgs<'a>,
    ) -> ProgramResult {
        let seeds = &[
            args.wallet.as_ref(),
            accounts.token_program.key.as_ref(),
            accounts.mint.key.as_ref(),
        ];

        Pda::validate(accounts.ata, seeds, &ASSOCIATED_TOKEN_PROGRAM_ID)?;

        let ix = match TokenProgram::detect_token_program(accounts.token_program)? {
            TokenProgram::Token => initialize_account3(
                accounts.token_program.key,
                accounts.ata.key,
                accounts.mint.key,
                args.wallet,
            )?,
            TokenProgram::Token2022 => {
                Instruction {
                    program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
                    accounts: vec![
                        AccountMeta::new(*accounts.payer.key, accounts.payer.is_signer),
                        AccountMeta::new(*accounts.ata.key, false),
                        AccountMeta::new_readonly(*args.wallet, false),
                        AccountMeta::new_readonly(*accounts.mint.key, false),
                        AccountMeta::new_readonly(*accounts.system_program.key, false),
                        AccountMeta::new_readonly(*accounts.token_program.key, false),
                        // AccountMeta::new_readonly(sysvar::rent::id(), false),
                    ],
                    data: vec![0],
                }
            }
        };

        invoke(
            &ix,
            &[
                accounts.payer.clone(),
                accounts.ata.clone(),
                accounts.mint.clone(),
                accounts.system_program.clone(),
                accounts.token_program.clone(),
                accounts.associated_token_program.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn init_if_needed<'a, 'info>(
        accounts: InitAssociatedTokenProgramAccounts<'a, 'info>,
        args: InitAssociatedTokenProgramArgs<'a>,
    ) -> ProgramResult {
        if UninitializedAccount::check(accounts.ata).is_ok() {
            Self::init(accounts, args)?;
        }

        Ok(())
    }

    pub fn check<'info>(
        ata: &AccountInfo<'info>,
        wallet: &Pubkey,
        mint: &Pubkey,
        token_program: &Pubkey,
    ) -> ProgramResult {
        let (expected_ata, _) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        if ata.key != &expected_ata {
            msg!("Invalid ATA seeds {}", ata.key);
            return Err(ProgramError::InvalidSeeds);
        }

        if ata.owner != token_program {
            msg!("Invalid ATA account {}", ata.key);
            return Err(ProgramError::InvalidAccountOwner);
        }

        let expected_len = if token_program == &TOKEN_2022_PROGRAM_ID {
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
    pub mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub associated_token_program: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
    pub ata: &'a AccountInfo<'info>,
}

pub struct InitAssociatedTokenProgramArgs<'a> {
    pub wallet: &'a Pubkey,
}

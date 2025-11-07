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
    Pda, TokenProgram, TOKEN_2022_PROGRAM_ID, TOKEN_ACCOUNT_2022_MIN_LEN, TOKEN_ACCOUNT_LEN,
};

pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub struct AssociatedTokenProgram;

impl AssociatedTokenProgram {
    pub fn init<'info>(
        payer: &AccountInfo<'info>,
        wallet: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        associated_token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        ata: &AccountInfo<'info>,
    ) -> ProgramResult {
        let seeds = &[
            wallet.key.as_ref(),
            token_program.key.as_ref(),
            mint.key.as_ref(),
        ];

        Pda::validate(ata, seeds, &ASSOCIATED_TOKEN_PROGRAM_ID)?;

        let ix = match TokenProgram::detect_token_program(token_program)? {
            TokenProgram::Token => {
                initialize_account3(token_program.key, ata.key, mint.key, wallet.key)?
            }
            TokenProgram::Token2022 => {
                Instruction {
                    program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
                    accounts: vec![
                        AccountMeta::new(*payer.key, payer.is_signer),
                        AccountMeta::new(*ata.key, false),
                        AccountMeta::new_readonly(*wallet.key, false),
                        AccountMeta::new_readonly(*mint.key, false),
                        AccountMeta::new_readonly(*system_program.key, false),
                        AccountMeta::new_readonly(*token_program.key, false),
                        // AccountMeta::new_readonly(sysvar::rent::id(), false),
                    ],
                    data: vec![0],
                }
            }
        };

        invoke(
            &ix,
            &[
                payer.clone(),
                ata.clone(),
                wallet.clone(),
                mint.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn init_if_needed<'info>(
        payer: &AccountInfo<'info>,
        wallet: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        associated_token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        ata: &AccountInfo<'info>,
    ) -> ProgramResult {
        if ata.lamports() == 0 || ata.data_is_empty() {
            Self::init(
                payer,
                wallet,
                mint,
                token_program,
                associated_token_program,
                system_program,
                ata,
            )?;
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

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey, pubkey::Pubkey,
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
        system_program: &AccountInfo<'info>,
        ata: &AccountInfo<'info>,
    ) -> ProgramResult {
        let seeds = &[
            wallet.key.as_ref(),
            token_program.key.as_ref(),
            mint.key.as_ref(),
        ];

        match TokenProgram::detect_token_program(token_program)? {
            TokenProgram::Token => {
                Pda::new(
                    payer,
                    ata,
                    system_program,
                    seeds,
                    TOKEN_ACCOUNT_LEN,
                    token_program.key,
                    &ASSOCIATED_TOKEN_PROGRAM_ID,
                )?
                .init_if_needed()?;

                let ix = initialize_account3(token_program.key, ata.key, mint.key, wallet.key)?;

                invoke(
                    &ix,
                    &[
                        ata.clone(),
                        mint.clone(),
                        wallet.clone(),
                        token_program.clone(),
                    ],
                )?;
            }
            TokenProgram::Token2022 => {
                Pda::new(
                    payer,
                    ata,
                    system_program,
                    seeds,
                    TOKEN_ACCOUNT_2022_MIN_LEN,
                    token_program.key,
                    &ASSOCIATED_TOKEN_PROGRAM_ID,
                )?
                .init_if_needed()?;
            }
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
            return Err(ProgramError::InvalidSeeds);
        }

        if ata.owner != token_program {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let expected_len = if token_program == &TOKEN_2022_PROGRAM_ID {
            TOKEN_ACCOUNT_2022_MIN_LEN..=usize::MAX
        } else {
            TOKEN_ACCOUNT_LEN..=TOKEN_ACCOUNT_LEN
        };

        if !expected_len.contains(&ata.data_len()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

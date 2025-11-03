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

use solana_program::{
    account_info::AccountInfo, msg, program::invoke_signed, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

#[derive(Debug)]
pub struct Pda<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub pda: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
    pub seeds: &'a [&'a [u8]],
    pub space: usize,
    pub program_id: &'a Pubkey,
    pub bump: u8,
}

impl<'a, 'info> Pda<'a, 'info> {
    pub fn new(
        accounts: InitPdaAccounts<'a, 'info>,
        args: InitPdaArgs<'a>,
    ) -> Result<Self, ProgramError> {
        let (_, bump) = Self::validate(accounts.pda, args.seeds, args.program_id)?;

        Ok(Self {
            payer: accounts.payer,
            pda: accounts.pda,
            system_program: accounts.system_program,
            seeds: args.seeds,
            space: args.space,
            program_id: args.program_id,
            bump,
        })
    }

    pub fn validate(
        pda: &'a AccountInfo<'info>,
        seeds: &'a [&'a [u8]],
        program_id: &'a Pubkey,
    ) -> Result<(Pubkey, u8), ProgramError> {
        let (derived_pda, bump) = Pubkey::find_program_address(seeds, program_id);
        if derived_pda != *pda.key {
            msg!("Invalid PDA: expected {}, got {}", derived_pda, pda.key);
            return Err(ProgramError::InvalidSeeds);
        }
        Ok((derived_pda, bump))
    }

    pub fn init(&self) -> Result<u8, ProgramError> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(self.space);

        let bump_slice: &[u8] = &[self.bump];
        let seed_vec: Vec<&[u8]> = self
            .seeds
            .iter()
            .copied()
            .chain(std::iter::once(bump_slice))
            .collect();
        let signer_seeds: &[&[&[u8]]] = &[&seed_vec];

        let ix = system_instruction::create_account(
            self.payer.key,
            self.pda.key,
            lamports,
            self.space as u64,
            self.program_id,
        );

        invoke_signed(
            &ix,
            &[
                self.payer.clone(),
                self.pda.clone(),
                self.system_program.clone(),
            ],
            signer_seeds,
        )?;

        Ok(self.bump)
    }
}

pub struct InitPdaAccounts<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub pda: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct InitPdaArgs<'a> {
    pub seeds: &'a [&'a [u8]],
    pub space: usize,
    pub program_id: &'a Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock::mock_account;

    #[test]
    fn test_valid_new_pda() {
        let payer = mock_account(Pubkey::new_unique(), false, false, 1, 0, Pubkey::default());
        let system_program = mock_account(Pubkey::default(), false, false, 1, 0, Pubkey::default());

        let seeds = &[b"test", payer.key.as_ref(), system_program.key.as_ref()];
        let (expected_ata, _) = Pubkey::find_program_address(seeds, &crate::ID);
        let pda = mock_account(expected_ata, false, true, 1, 0, crate::ID);

        let accounts = InitPdaAccounts {
            payer: &payer,
            pda: &pda,
            system_program: &system_program,
        };
        let args = InitPdaArgs {
            seeds,
            space: 0,
            program_id: &crate::ID,
        };

        assert!(Pda::new(accounts, args).is_ok());
    }

    #[test]
    fn test_invalid_new_pda() {
        let payer = mock_account(Pubkey::new_unique(), false, false, 1, 0, Pubkey::default());
        let system_program = mock_account(Pubkey::default(), false, false, 1, 0, Pubkey::default());

        let seeds = &[b"test", payer.key.as_ref(), system_program.key.as_ref()];
        let (expected_ata, _) = Pubkey::find_program_address(seeds, &crate::ID);
        let pda = mock_account(expected_ata, false, true, 1, 0, crate::ID);

        let accounts = InitPdaAccounts {
            payer: &payer,
            pda: &pda,
            system_program: &system_program,
        };
        let args = InitPdaArgs {
            seeds: &[],
            space: 0,
            program_id: &crate::ID,
        };

        assert_eq!(
            Pda::new(accounts, args).unwrap_err(),
            ProgramError::InvalidSeeds
        );
    }
}

use solana_program::{
    account_info::AccountInfo, msg, program::invoke_signed, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

pub struct Pda<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub pda: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
    pub seeds: &'a [&'a [u8]],
    pub space: usize,
    pub owner: &'a Pubkey,
    pub bump: u8,
}

impl<'a, 'info> Pda<'a, 'info> {
    pub fn new(
        payer: &'a AccountInfo<'info>,
        pda: &'a AccountInfo<'info>,
        system_program: &'a AccountInfo<'info>,
        seeds: &'a [&'a [u8]],
        space: usize,
        owner: &'a Pubkey,
        program_id: &'a Pubkey,
    ) -> Result<Self, ProgramError> {
        let (derived_pda, bump) = Pubkey::find_program_address(seeds, program_id);
        if derived_pda != *pda.key {
            msg!("Invalid PDA: expected {}, got {}", derived_pda, pda.key);
            return Err(ProgramError::InvalidSeeds);
        }

        Ok(Self {
            payer,
            pda,
            system_program,
            seeds,
            space,
            owner,
            bump,
        })
    }

    pub fn init(&self) -> Result<u8, ProgramError> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(self.space);

        let bump_slice: &[u8] = &[self.bump];
        let mut signer_seed_vec = Vec::with_capacity(self.seeds.len() + 1);
        signer_seed_vec.extend_from_slice(self.seeds);
        signer_seed_vec.push(bump_slice);
        let signer_seeds: &[&[&[u8]]] = &[signer_seed_vec.as_slice()];

        invoke_signed(
            &system_instruction::create_account(
                self.payer.key,
                self.pda.key,
                lamports,
                self.space as u64,
                self.owner,
            ),
            &[
                self.payer.clone(),
                self.pda.clone(),
                self.system_program.clone(),
            ],
            signer_seeds,
        )?;

        Ok(self.bump)
    }

    pub fn init_if_needed(&self) -> Result<u8, ProgramError> {
        if self.pda.lamports() == 0 || self.pda.data_is_empty() {
            self.init()?;
        }

        Ok(self.bump)
    }
}

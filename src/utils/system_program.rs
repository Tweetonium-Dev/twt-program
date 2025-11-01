use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, system_instruction,
};

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
}

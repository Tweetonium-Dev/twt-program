use solana_program::entrypoint::ProgramResult;

pub trait ProcessInstruction {
    fn process(self) -> ProgramResult;
}

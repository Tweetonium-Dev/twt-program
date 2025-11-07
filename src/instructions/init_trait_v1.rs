use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{InitTraitItemArgs, TraitItem, MAX_ROYALTY_RECIPIENTS},
    utils::{
        AccountCheck, InitMplCoreCollectionArgs, InitPdaArgs, MplCoreProgram, ProcessInstruction,
        SignerAccount, SystemProgram, UninitializedAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct InitTraitV1Accounts<'a, 'info> {
    /// Authority that will control trait (e.g. protocol wallet).
    /// Must be a signer.
    pub authority: &'a AccountInfo<'info>,

    /// PDA: `[program_id, trait_collection, "config"]` — stores `Config` struct.
    /// Must be uninitialized, writable, owned by this program.
    pub trait_pda: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this trait.
    /// Must be initialized before trait creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub trait_collection: &'a AccountInfo<'info>,

    /// System program — required for PDA creation and rent.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for InitTraitV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, trait_pda, trait_collection, system_program, mpl_core] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;
        SignerAccount::check(trait_collection)?;

        WritableAccount::check(trait_pda)?;

        UninitializedAccount::check(trait_collection)?;
        // FIXME: Uncomment this on mainnet
        // UninitializedAccount::check(trait_pda)?;

        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        Ok(Self {
            authority,
            trait_pda,
            trait_collection,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitTraitV1InstructionData {
    pub max_supply: u64,
    pub mint_fee_lamports: u64,
    pub trait_name: String,
    pub trait_uri: String,
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
    pub royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
}

#[derive(Debug)]
pub struct InitTraitV1<'a, 'info> {
    pub accounts: InitTraitV1Accounts<'a, 'info>,
    pub instruction_data: InitTraitV1InstructionData,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info> InitTraitV1<'a, 'info> {
    fn check_trait_royalties(&self) -> ProgramResult {
        TraitItem::check_trait_royalties(
            self.instruction_data.num_royalty_recipients,
            self.instruction_data.royalty_shares_bps,
        )
    }

    fn init_trait(&self) -> ProgramResult {
        let mut trait_data = self.accounts.trait_pda.try_borrow_mut_data()?;
        let seeds: &[&[u8]] = &[TraitItem::SEED, self.accounts.trait_collection.key.as_ref()];

        TraitItem::init_if_needed(
            &mut trait_data,
            InitPdaArgs {
                payer: self.accounts.authority,
                pda: self.accounts.trait_pda,
                system_program: self.accounts.system_program,
                seeds,
                space: TraitItem::LEN,
                program_id: self.program_id,
            },
            InitTraitItemArgs {
                authority: *self.accounts.authority.key,
                max_supply: self.instruction_data.max_supply,
                user_minted: 0,
                mint_fee_lamports: self.instruction_data.mint_fee_lamports,
            },
        )
    }

    fn init_collection(self) -> ProgramResult {
        MplCoreProgram::init_collection(
            self.accounts.trait_collection,
            self.accounts.authority,
            self.accounts.mpl_core,
            self.accounts.system_program,
            InitMplCoreCollectionArgs {
                num_royalty_recipients: self.instruction_data.num_royalty_recipients,
                royalty_recipients: self.instruction_data.royalty_recipients,
                royalty_shares_bps: self.instruction_data.royalty_shares_bps,
                name: self.instruction_data.trait_name,
                uri: self.instruction_data.trait_uri,
            },
        )
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        InitTraitV1InstructionData,
        &'a Pubkey,
    )> for InitTraitV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            InitTraitV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = InitTraitV1Accounts::try_from(accounts)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
        })
    }
}

impl<'a, 'info> ProcessInstruction for InitTraitV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.check_trait_royalties()?;
        self.init_trait()?;
        self.init_collection()
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::TraitItem,
    utils::{
        AccountCheck, InitMplCoreAssetArgs, MplCoreProgram, Pda, ProcessInstruction, SignerAccount,
        SystemProgram, UninitializedAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct MintTraitV1Accounts<'a, 'info> {
    /// User paying the mint price in solana.
    /// Must be signer.
    pub payer: &'a AccountInfo<'info>,

    /// PDA: `[program_id, trait_collection, "config"]` — stores `Config` struct.
    /// Must be uninitialized, writable, owned by this program.
    pub trait_pda: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this trait.
    /// Must be initialized before trait creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub trait_collection: &'a AccountInfo<'info>,

    /// Trait asset (MPL Core) — the NFT being minted.
    /// Must be uninitialized, owned by `mpl_core`.
    pub trait_asset: &'a AccountInfo<'info>,

    /// Protocol wallet — receives the configurable SOL protocol fee.
    /// Must writable, not zero address, owned by system_program.
    pub protocol_wallet: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for MintTraitV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [payer, trait_pda, trait_collection, trait_asset, protocol_wallet, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;
        SignerAccount::check(trait_asset)?;

        WritableAccount::check(trait_collection)?;
        WritableAccount::check(protocol_wallet)?;

        UninitializedAccount::check(trait_asset)?;

        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        Ok(Self {
            payer,
            trait_pda,
            trait_collection,
            trait_asset,
            protocol_wallet,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MinTraitV1InstructionData {
    pub nft_name: String,
    pub nft_uri: String,
}

#[derive(Debug)]
pub struct MintTraitV1<'a, 'info> {
    pub accounts: MintTraitV1Accounts<'a, 'info>,
    pub instruction_data: MinTraitV1InstructionData,
}

impl<'a, 'info> MintTraitV1<'a, 'info> {
    fn check_mint_eligibility(&self, trait_item: &TraitItem) -> ProgramResult {
        if trait_item.stock_available() {
            msg!(
                "All trait are minted. Allowed supply: {}. Minted {}",
                trait_item.max_supply,
                trait_item.user_minted,
            );
            return Err(ProgramError::Custom(0));
        }

        Ok(())
    }

    fn pay_protocol_fee(&self, trait_item: &TraitItem) -> ProgramResult {
        if trait_item.is_free_mint_fee() {
            return Ok(());
        }

        SystemProgram::transfer(
            self.accounts.payer,
            self.accounts.protocol_wallet,
            self.accounts.system_program,
            trait_item.mint_fee_lamports,
        )
    }

    fn mint_nft(self, trait_item: &mut TraitItem) -> ProgramResult {
        MplCoreProgram::create(
            self.accounts.trait_asset,
            self.accounts.trait_collection,
            self.accounts.payer,
            None,
            self.accounts.mpl_core,
            self.accounts.system_program,
            InitMplCoreAssetArgs {
                name: self.instruction_data.nft_name,
                uri: self.instruction_data.nft_uri,
            },
        )?;

        trait_item.increment_user_minted()?;

        Ok(())
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        MinTraitV1InstructionData,
        &'a Pubkey,
    )> for MintTraitV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            MinTraitV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = MintTraitV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.trait_pda,
            &[TraitItem::SEED, accounts.trait_collection.key.as_ref()],
            program_id,
        )?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a, 'info> ProcessInstruction for MintTraitV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut trait_data = self.accounts.trait_pda.try_borrow_mut_data()?;
        let trait_item = TraitItem::load_mut(trait_data.as_mut())?;

        self.check_mint_eligibility(trait_item)?;
        self.pay_protocol_fee(trait_item)?;
        self.mint_nft(trait_item)?;

        msg!("MintTraitV1: minted trait NFT");
        Ok(())
    }
}

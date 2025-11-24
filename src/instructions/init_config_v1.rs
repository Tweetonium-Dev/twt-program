use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{ConfigV1, InitConfigAccounts, InitConfigArgs, NftAuthorityV1, VestingMode},
    utils::{
        AccountCheck, InitMplCoreCollectionAccounts, InitMplCoreCollectionArgs, InitPdaAccounts,
        InitPdaArgs, MintAccount, MplCoreProgram, Pda, ProcessInstruction, SignerAccount,
        SystemProgram, TokenProgram, UninitializedAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct InitConfigV1Accounts<'a, 'info> {
    /// Authority that will control config updates (e.g. admin wallet).
    /// Must be a signer.
    pub admin: &'a AccountInfo<'info>,

    /// PDA: `["config_v1", nft_collection, token_mint, program_id]` — stores global config.
    /// Must be uninitialized, writable, owned by this program.
    pub config_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "nft_authority"]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be signer and initialized before nft creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// Token mint (fungible token used for minting/refunding e.g. ZDLT).
    /// Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022.
    pub token_mint: &'a AccountInfo<'info>,

    /// System program — required for PDA creation and rent.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for InitConfigV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [admin, config_pda, nft_authority, nft_collection, token_mint, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(admin)?;
        SignerAccount::check(nft_collection)?;

        WritableAccount::check(config_pda)?;

        UninitializedAccount::check(nft_collection)?;

        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        Ok(Self {
            admin,
            config_pda,
            nft_authority,
            nft_collection,
            token_mint,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitConfigV1InstructionData {
    pub max_supply: u64,
    pub released: u64,
    pub max_mint_per_user: u64,
    pub max_mint_per_vip_user: u64,
    pub vesting_mode: VestingMode,
    pub vesting_unlock_ts: i64,
    pub mint_nft_fee_lamports: u64,
    pub update_nft_fee_lamports: u64,
    pub mint_price_total: u64,
    pub escrow_amount: u64,
    pub num_revenue_wallets: u8,
    pub revenue_wallets: [Pubkey; 5],
    pub revenue_shares: [u64; 5],
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; 5],
    pub royalty_shares_bps: [u16; 5],
    pub collection_name: String,
    pub collection_uri: String,
}

#[derive(Debug)]
pub struct InitConfigV1<'a, 'info> {
    pub accounts: InitConfigV1Accounts<'a, 'info>,
    pub instruction_data: InitConfigV1InstructionData,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        InitConfigV1InstructionData,
        &'a Pubkey,
    )> for InitConfigV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            InitConfigV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = InitConfigV1Accounts::try_from(accounts)?;

        Pda::validate(accounts.nft_authority, &[NftAuthorityV1::SEED], program_id)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
        })
    }
}

impl<'a, 'info> InitConfigV1<'a, 'info> {
    fn check_config_data(&self) -> ProgramResult {
        ConfigV1::check_revenue_wallets(
            self.instruction_data.mint_price_total,
            self.instruction_data.escrow_amount,
            self.instruction_data.num_revenue_wallets,
            self.instruction_data.revenue_wallets,
            self.instruction_data.revenue_shares,
        )?;
        ConfigV1::check_nft_royalties(
            self.instruction_data.num_royalty_recipients,
            self.instruction_data.royalty_recipients,
            self.instruction_data.royalty_shares_bps,
        )
    }

    fn init_config(&self) -> ProgramResult {
        let seeds: &[&[u8]] = &[
            ConfigV1::SEED,
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
        ];
        let decimals = TokenProgram::get_decimal(self.accounts.token_mint)?;

        ConfigV1::init_if_needed(
            InitConfigAccounts {
                pda: self.accounts.config_pda,
            },
            InitConfigArgs {
                admin: *self.accounts.admin.key,
                mint: *self.accounts.token_mint.key,
                max_supply: self.instruction_data.max_supply,
                released: self.instruction_data.released,
                max_mint_per_user: self.instruction_data.max_mint_per_user,
                max_mint_per_vip_user: self.instruction_data.max_mint_per_vip_user,
                mint_price_total: self.instruction_data.mint_price_total,
                admin_minted: 0,
                user_minted: 0,
                vesting_mode: self.instruction_data.vesting_mode,
                vesting_unlock_ts: self.instruction_data.vesting_unlock_ts,
                mint_nft_fee_lamports: self.instruction_data.mint_nft_fee_lamports,
                update_nft_fee_lamports: self.instruction_data.update_nft_fee_lamports,
                escrow_amount: self.instruction_data.escrow_amount,
                mint_decimals: decimals,
                num_revenue_wallets: self.instruction_data.num_revenue_wallets,
                revenue_wallets: self.instruction_data.revenue_wallets,
                revenue_shares: self.instruction_data.revenue_shares,
            },
            InitPdaAccounts {
                payer: self.accounts.admin,
                pda: self.accounts.config_pda,
                system_program: self.accounts.system_program,
            },
            InitPdaArgs {
                seeds,
                space: ConfigV1::LEN,
                program_id: self.program_id,
            },
        )
    }

    fn init_collection(self) -> ProgramResult {
        MplCoreProgram::init_collection(
            InitMplCoreCollectionAccounts {
                payer: self.accounts.admin,
                collection: self.accounts.nft_collection,
                update_authority: Some(self.accounts.nft_authority),
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            InitMplCoreCollectionArgs {
                num_royalty_recipients: self.instruction_data.num_royalty_recipients,
                royalty_recipients: self.instruction_data.royalty_recipients,
                royalty_shares_bps: self.instruction_data.royalty_shares_bps,
                name: self.instruction_data.collection_name,
                uri: self.instruction_data.collection_uri,
            },
        )
    }
}

impl<'a, 'info> ProcessInstruction for InitConfigV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.check_config_data()?;
        self.init_config()?;
        self.init_collection()
    }
}

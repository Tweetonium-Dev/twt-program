use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{Config, InitVaultArgs, NftAuthority, UserMinted, Vault},
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenProgram,
        ConfigAccount, InitMplCoreAssetArgs, InitPdaArgs, MintAccount, MplCoreProgram, Pda,
        ProcessInstruction, SignerAccount, SystemProgram, TokenProgram, TokenTransferArgs,
        UninitializedAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct MintVipV1Accounts<'a, 'info> {
    /// User paying the mint price in 'token_mint' and solana.
    /// Must be signer and owner of `payer_ata`.
    pub payer: &'a AccountInfo<'info>,

    /// PDA: `[program_id, token_mint, nft_collection, "config"]` — stores global config.
    /// Must be readable, owned by program.
    pub config_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, authority, token_mint, nft_collection, "vault"]` — stores `Vault` state.
    /// Must be writable if updating vault balance.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Associated Token Account (ATA) of the vault PDA.
    /// Holds 'token_mint' received from users.
    /// Must be writable, owned by `token_program`.
    pub vault_ata: &'a AccountInfo<'info>,

    /// Payer's ATA for 'token_mint' — source of payment.
    /// Must be writable, owned by `token_program`.
    pub payer_ata: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "minted", payer.key]` — per-user mint flag.
    /// Prevents double-minting.
    /// Must be uninitialized or checked for prior mint.
    pub user_minted_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "nft_authority"]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be initialized before config creation via `CreateV1CpiBuilder`.
    /// Used as part of the config PDA seeds: `[program_id, token_mint, collection.key.as_ref()]`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// NFT asset (MPL Core) — the NFT being minted.
    /// Must be uninitialized, owned by `mpl_core`.
    pub nft_asset: &'a AccountInfo<'info>,

    /// Token mint — the token being escrowed (e.g. ZDLT.
    /// Must match `config_pda.data.mint`, owned by `token_program`.
    pub token_mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_mint.owner`.
    pub token_program: &'a AccountInfo<'info>,

    /// Associated Token Program (ATA).
    /// Used to derive and create ATAs (`vault_ata`, `payer_ata`) deterministically.
    /// Must be the official SPL Associated Token Account program.
    pub associated_token_program: &'a AccountInfo<'info>,

    /// Protocol wallet — receives the configurable SOL protocol fee.
    /// Must writable, not zero address, owned by system_program.
    pub protocol_wallet: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,

    /// Revenue wallets accounts: ATAs for the additional wallets defined in config
    /// Must match config.dao_wallets, config.dao_prices, and config.num_dao_wallet.
    pub revenue_wallets: &'a [AccountInfo<'info>],
}

impl<'a, 'info> MintVipV1Accounts<'a, 'info> {
    const ACCOUNTS_LEN: usize = 15;
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for MintVipV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        if accounts.len() < Self::ACCOUNTS_LEN {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let payer = &accounts[0];
        let config_pda = &accounts[1];
        let vault_pda = &accounts[2];
        let vault_ata = &accounts[3];
        let payer_ata = &accounts[4];
        let user_minted_pda = &accounts[5];
        let nft_authority = &accounts[6];
        let nft_collection = &accounts[7];
        let nft_asset = &accounts[8];
        let token_mint = &accounts[9];
        let token_program = &accounts[10];
        let associated_token_program = &accounts[11];
        let protocol_wallet = &accounts[12];
        let system_program = &accounts[13];
        let mpl_core = &accounts[14];
        let revenue_wallets = &accounts[Self::ACCOUNTS_LEN..];

        SignerAccount::check(payer)?;
        SignerAccount::check(nft_asset)?;

        WritableAccount::check(config_pda)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;
        WritableAccount::check(user_minted_pda)?;
        WritableAccount::check(nft_collection)?;
        WritableAccount::check(protocol_wallet)?;

        UninitializedAccount::check(vault_pda)?;
        UninitializedAccount::check(nft_asset)?;

        ConfigAccount::check(config_pda)?;
        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        AssociatedTokenAccount::check(payer_ata, payer.key, token_mint.key, token_program.key)?;

        Ok(Self {
            payer,
            config_pda,
            vault_pda,
            vault_ata,
            payer_ata,
            user_minted_pda,
            nft_authority,
            nft_collection,
            nft_asset,
            token_mint,
            token_program,
            associated_token_program,
            protocol_wallet,
            system_program,
            mpl_core,
            revenue_wallets,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MintVipV1InstructionData {
    pub nft_name: String,
    pub nft_uri: String,
}

#[derive(Debug)]
pub struct MintVipV1<'a, 'info> {
    pub accounts: MintVipV1Accounts<'a, 'info>,
    pub instruction_data: MintVipV1InstructionData,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info> MintVipV1<'a, 'info> {
    fn check_mint_eligibility(&self, config: &Config) -> ProgramResult {
        let max_supply = config.max_supply;
        let released = config.released;
        let admin_minted = config.admin_minted;
        let user_minted = config.user_minted;
        let minted = admin_minted + user_minted;

        if config.nft_stock_available() {
            msg!(
                "All nft are minted. Allowed supply: {}. Minted {}",
                max_supply,
                minted,
            );
            return Err(ProgramError::Custom(0));
        }

        if config.user_mint_available() {
            msg!(
                "Sold out. Allowed supply: {}. Minted: {}",
                released,
                user_minted
            );
            return Err(ProgramError::Custom(1));
        }

        Ok(())
    }

    fn init_user_minted_if_needed(&self) -> ProgramResult {
        let mut minted_user_data = self.accounts.user_minted_pda.try_borrow_mut_data()?;

        let seeds = &[
            UserMinted::SEED,
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
            self.accounts.payer.key.as_ref(),
        ];

        UserMinted::init_if_needed(
            &mut minted_user_data,
            InitPdaArgs {
                payer: self.accounts.payer,
                pda: self.accounts.user_minted_pda,
                system_program: self.accounts.system_program,
                seeds,
                space: UserMinted::LEN,
                program_id: self.program_id,
            },
            self.accounts.payer.key,
        )
    }

    fn tf_to_revenue_wallet(&self, config: &Config, index: usize) -> ProgramResult {
        let dao_wallet_ata = &self
            .accounts
            .revenue_wallets
            .get(index)
            .ok_or(ProgramError::InvalidInstructionData)
            .inspect_err(|_| msg!("DAO wallet ata index {} not found", index))?;
        let dao_wallet_key = &config
            .dao_wallet(index)
            .inspect_err(|_| msg!("DAO wallet index {} not found!", index))?;
        let dao_price = config
            .dao_price(index)
            .inspect_err(|_| msg!("DAO prices index {} not found!", index))?;

        AssociatedTokenAccount::check(
            dao_wallet_ata,
            dao_wallet_key,
            self.accounts.token_mint.key,
            self.accounts.token_program.key,
        )?;
        WritableAccount::check(dao_wallet_ata)?;

        if config.allow_tf_to_dao_wallet(index) {
            TokenProgram::transfer(TokenTransferArgs {
                source: self.accounts.payer_ata,
                destination: dao_wallet_ata,
                authority: self.accounts.payer,
                mint: self.accounts.token_mint,
                token_program: self.accounts.token_program,
                signer_pubkeys: &[],
                amount: dao_price,
                decimals: config.mint_decimals,
            })?;
        }

        Ok(())
    }

    fn pay_to_all_revenue_wallets(&self, config: &Config) -> ProgramResult {
        if self.accounts.revenue_wallets.len() != config.num_revenue_wallets as usize {
            msg!("Incorrect number of remaining accounts for revenue's wallet ATAs");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        for wallet_idx in 0..config.num_revenue_wallets as usize {
            self.tf_to_revenue_wallet(config, wallet_idx)?;
        }

        Ok(())
    }

    fn store_to_vault(&self, config: &Config) -> ProgramResult {
        if !config.need_vault() {
            return Ok(());
        }

        let mut vault_data = self.accounts.vault_pda.try_borrow_mut_data()?;

        let seeds: &[&[u8]] = &[
            Vault::SEED,
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
            self.accounts.payer.key.as_ref(),
        ];

        Vault::init_if_needed(
            &mut vault_data,
            InitPdaArgs {
                payer: self.accounts.payer,
                pda: self.accounts.vault_pda,
                system_program: self.accounts.system_program,
                seeds,
                space: Vault::LEN,
                program_id: self.program_id,
            },
            InitVaultArgs {
                owner: *self.accounts.payer.key,
                nft: *self.accounts.nft_asset.key,
                amount: config.escrow_amount,
                is_unlocked: false,
            },
        )?;

        AssociatedTokenProgram::init_if_needed(
            self.accounts.payer,
            self.accounts.vault_pda,
            self.accounts.token_mint,
            self.accounts.token_program,
            self.accounts.associated_token_program,
            self.accounts.system_program,
            self.accounts.vault_ata,
        )?;

        TokenProgram::transfer(TokenTransferArgs {
            source: self.accounts.payer_ata,
            destination: self.accounts.vault_ata,
            authority: self.accounts.payer,
            mint: self.accounts.token_mint,
            token_program: self.accounts.token_program,
            signer_pubkeys: &[],
            amount: config.escrow_amount,
            decimals: config.mint_decimals,
        })?;

        Ok(())
    }

    fn pay_protocol_fee(&self, config: &Config) -> ProgramResult {
        if config.is_free_mint_fee() {
            return Ok(());
        }

        SystemProgram::transfer(
            self.accounts.payer,
            self.accounts.protocol_wallet,
            self.accounts.system_program,
            config.mint_fee_lamports,
        )
    }

    fn mint_nft(self, config: &mut Config, user_minted: &mut UserMinted) -> ProgramResult {
        MplCoreProgram::create(
            self.accounts.nft_asset,
            self.accounts.nft_collection,
            self.accounts.payer,
            Some(self.accounts.nft_authority),
            self.accounts.mpl_core,
            self.accounts.system_program,
            InitMplCoreAssetArgs {
                name: self.instruction_data.nft_name,
                uri: self.instruction_data.nft_uri,
            },
        )?;

        user_minted.increment();
        config.increment_user_minted()?;

        Ok(())
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        MintVipV1InstructionData,
        &'a Pubkey,
    )> for MintVipV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            MintVipV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = MintVipV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.config_pda,
            &[
                Config::SEED,
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        Pda::validate(accounts.nft_authority, &[NftAuthority::SEED], program_id)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
        })
    }
}

impl<'a, 'info> ProcessInstruction for MintVipV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut config_data = self.accounts.config_pda.try_borrow_mut_data()?;
        let config = Config::load_mut(config_data.as_mut())?;

        self.init_user_minted_if_needed()?;

        let mut minted_user_data = self.accounts.user_minted_pda.try_borrow_mut_data()?;
        let user_minted = UserMinted::load_mut(minted_user_data.as_mut())?;
        if user_minted.has_reached_vip_limit(config) {
            msg!("User VIP has minted their allowed supply");
            return Err(ProgramError::Custom(2));
        }

        self.check_mint_eligibility(config)?;
        self.store_to_vault(config)?;
        self.pay_to_all_revenue_wallets(config)?;
        self.pay_protocol_fee(config)?;
        self.mint_nft(config, user_minted)?;

        msg!(
            "MintVipV1: minted NFT and escrowed {} tokens",
            config.escrow_amount
        );

        Ok(())
    }
}

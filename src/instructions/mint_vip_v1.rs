use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{
        ConfigV1, InitUserMintedAccounts, InitUserMintedArgs, InitVaultAccounts, InitVaultArgs,
        NftAuthorityV1, UserMintedV1, Vault,
    },
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenProgram,
        ConfigAccount, CreateMplCoreAssetAccounts, CreateMplCoreAssetArgs,
        InitAssociatedTokenProgramAccounts, InitPdaAccounts, InitPdaArgs, MintAccount,
        MplCoreProgram, Pda, ProcessInstruction, RevenueWallet, RevenueWalletAccounts,
        RevenueWalletArgs, SignerAccount, SystemProgram, TokenProgram, TokenTransferAccounts,
        TokenTransferArgs, UninitializedAccount, WritableAccount,
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

    /// PDA: `[program_id, token_mint, nft_collection, nft_asset, "vault"]` — stores `Vault` state.
    /// Must be writable if updating vault balance.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Associated Token Account (ATA) of the vault PDA.
    /// Holds 'token_mint' received from users.
    /// Must be writable, owned by `token_program`.
    pub vault_ata: &'a AccountInfo<'info>,

    /// Payer's ATA for 'token_mint' — source of payment.
    /// Must be writable, owned by `token_program`.
    pub payer_ata: &'a AccountInfo<'info>,

    /// PDA: `[program_id, payer, token_mint, nft_collection, "minted"]` — per-user mint flag.
    /// Prevents double-minting.
    /// Must be uninitialized or checked for prior mint.
    pub user_minted_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "nft_authority"]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be initialized before config creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// NFT asset (MPL Core) — the NFT being minted.
    /// Must be uninitialized, owned by `mpl_core`.
    pub nft_asset: &'a AccountInfo<'info>,

    /// Token mint — the token being escrowed (e.g. ZDLT.
    /// Must match `config_pda.data.mint`, owned by `token_program`.
    pub token_mint: &'a AccountInfo<'info>,

    // ---------------- Revenue Wallets ----------------
    /// Revenue wallet #0 — corresponds to `config.revenue_wallet(0)`.
    /// Must be writable if receiving transfer.
    pub revenue_wallet_0: &'a AccountInfo<'info>,
    pub revenue_wallet_ata_0: &'a AccountInfo<'info>,

    /// Revenue wallet #1 — corresponds to `config.revenue_wallet(1)`.
    /// Must be writable if receiving transfer.
    pub revenue_wallet_1: &'a AccountInfo<'info>,
    pub revenue_wallet_ata_1: &'a AccountInfo<'info>,

    /// Revenue wallet #2 — corresponds to `config.revenue_wallet(2)`.
    /// Must be writable if receiving transfer.
    pub revenue_wallet_2: &'a AccountInfo<'info>,
    pub revenue_wallet_ata_2: &'a AccountInfo<'info>,

    /// Revenue wallet #3 — corresponds to `config.revenue_wallet(3)`.
    /// Must be writable if receiving transfer.
    pub revenue_wallet_3: &'a AccountInfo<'info>,
    pub revenue_wallet_ata_3: &'a AccountInfo<'info>,

    /// Revenue wallet #4 — corresponds to `config.revenue_wallet(4)`.
    /// Must be writable if receiving transfer.
    pub revenue_wallet_4: &'a AccountInfo<'info>,
    pub revenue_wallet_ata_4: &'a AccountInfo<'info>,

    // --------------------------------------------------
    /// Protocol wallet — receives the configurable SOL protocol fee.
    /// Must writable, not zero address, owned by system_program.
    pub protocol_wallet: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_mint.owner`.
    pub token_program: &'a AccountInfo<'info>,

    /// Associated Token Program (ATA).
    /// Used to derive and create ATAs (`vault_ata`, `payer_ata`) deterministically.
    /// Must be the official SPL Associated Token Account program.
    pub associated_token_program: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for MintVipV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [payer, config_pda, vault_pda, vault_ata, payer_ata, user_minted_pda, nft_authority, nft_collection, nft_asset, token_mint, revenue_wallet_0, revenue_wallet_ata_0, revenue_wallet_1, revenue_wallet_ata_1, revenue_wallet_2, revenue_wallet_ata_2, revenue_wallet_3, revenue_wallet_ata_3, revenue_wallet_4, revenue_wallet_ata_4, protocol_wallet, token_program, associated_token_program, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;
        SignerAccount::check(nft_asset)?;

        WritableAccount::check(config_pda)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;
        WritableAccount::check(user_minted_pda)?;
        WritableAccount::check(nft_collection)?;
        WritableAccount::check(nft_asset)?;
        WritableAccount::check(protocol_wallet)?;

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
            revenue_wallet_0,
            revenue_wallet_ata_0,
            revenue_wallet_1,
            revenue_wallet_ata_1,
            revenue_wallet_2,
            revenue_wallet_ata_2,
            revenue_wallet_3,
            revenue_wallet_ata_3,
            revenue_wallet_4,
            revenue_wallet_ata_4,
            protocol_wallet,
            token_program,
            associated_token_program,
            system_program,
            mpl_core,
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
    pub nft_authority_bump: u8,
}

impl<'a, 'info> MintVipV1<'a, 'info> {
    fn check_mint_eligibility(&self, config: &ConfigV1) -> ProgramResult {
        let max_supply = config.max_supply;
        let released = config.released;
        let admin_minted = config.admin_minted;
        let user_minted = config.user_minted;
        let minted = admin_minted + user_minted;

        if !config.nft_stock_available() {
            msg!(
                "All nft are minted. Allowed supply: {}. Minted {}",
                max_supply,
                minted,
            );
            return Err(ProgramError::Custom(0));
        }

        if !config.user_mint_available() {
            msg!(
                "Sold out. Allowed supply: {}. Minted: {}",
                released,
                user_minted
            );
            return Err(ProgramError::Custom(1));
        }

        Ok(())
    }

    fn init_user_mint_if_needed(&self) -> ProgramResult {
        let seeds = &[
            UserMintedV1::SEED,
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
            self.accounts.payer.key.as_ref(),
        ];

        UserMintedV1::init_if_needed(
            InitUserMintedAccounts {
                pda: self.accounts.user_minted_pda,
            },
            InitUserMintedArgs {
                owner: self.accounts.payer.key,
            },
            InitPdaAccounts {
                payer: self.accounts.payer,
                pda: self.accounts.user_minted_pda,
                system_program: self.accounts.system_program,
            },
            InitPdaArgs {
                seeds,
                space: UserMintedV1::LEN,
                program_id: self.program_id,
            },
        )
    }

    fn store_to_vault(&self, config: &ConfigV1) -> ProgramResult {
        if !config.need_vault() {
            return Ok(());
        }

        let seeds: &[&[u8]] = &[
            Vault::SEED,
            self.accounts.nft_asset.key.as_ref(),
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
        ];

        Vault::init_if_needed(
            InitVaultAccounts {
                pda: self.accounts.vault_pda,
            },
            InitVaultArgs {
                owner: *self.accounts.payer.key,
                nft: *self.accounts.nft_asset.key,
                amount: config.escrow_amount,
                is_unlocked: false,
            },
            InitPdaAccounts {
                payer: self.accounts.payer,
                pda: self.accounts.vault_pda,
                system_program: self.accounts.system_program,
            },
            InitPdaArgs {
                seeds,
                space: Vault::LEN,
                program_id: self.program_id,
            },
        )?;

        AssociatedTokenProgram::init_if_needed(InitAssociatedTokenProgramAccounts {
            payer: self.accounts.payer,
            wallet: self.accounts.vault_pda,
            mint: self.accounts.token_mint,
            token_program: self.accounts.token_program,
            associated_token_program: self.accounts.associated_token_program,
            system_program: self.accounts.system_program,
            ata: self.accounts.vault_ata,
        })?;

        TokenProgram::transfer(
            TokenTransferAccounts {
                source: self.accounts.payer_ata,
                destination: self.accounts.vault_ata,
                authority: self.accounts.payer,
                mint: self.accounts.token_mint,
                token_program: self.accounts.token_program,
            },
            TokenTransferArgs {
                signer_pubkeys: &[],
                amount: config.escrow_amount,
                decimals: config.mint_decimals,
            },
        )?;

        Ok(())
    }

    fn pay_to_all_revenue_wallets(&self, config: &ConfigV1) -> ProgramResult {
        let num_wallets = config.num_revenue_wallets as usize;

        if num_wallets == 0 {
            return Ok(());
        }

        let revenue_wallets = [
            self.accounts.revenue_wallet_0,
            self.accounts.revenue_wallet_1,
            self.accounts.revenue_wallet_2,
            self.accounts.revenue_wallet_3,
            self.accounts.revenue_wallet_4,
        ];

        let revenue_wallet_atas = [
            self.accounts.revenue_wallet_ata_0,
            self.accounts.revenue_wallet_ata_1,
            self.accounts.revenue_wallet_ata_2,
            self.accounts.revenue_wallet_ata_3,
            self.accounts.revenue_wallet_ata_4,
        ];

        if num_wallets > revenue_wallet_atas.len() {
            msg!("Incorrect number of accounts for revenue's wallet ATAs");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        for index in 0..num_wallets {
            let (Ok(expected_revenue_wallet), Ok(amount)) = (
                config
                    .revenue_wallet(index)
                    .inspect_err(|_| msg!("Revenue wallet index {} not found!", index)),
                config
                    .revenue_share(index)
                    .inspect_err(|_| msg!("Revenue share index {} not found!", index)),
            ) else {
                continue;
            };

            let revenue_wallet = revenue_wallets
                .get(index)
                .ok_or(ProgramError::NotEnoughAccountKeys)
                .inspect_err(|_| msg!("Missing revenue wallet at index {}", index))?;
            let revenue_ata = revenue_wallet_atas
                .get(index)
                .ok_or(ProgramError::InvalidAccountData)
                .inspect_err(|_| msg!("Missing revenue wallet ATA at index {}", index))?;

            if revenue_wallet.key != expected_revenue_wallet {
                msg!(
                    "Revenue wallet mismatch at index {}. Expected {}, got {}",
                    index,
                    expected_revenue_wallet,
                    revenue_wallet.key
                );
                return Err(ProgramError::InvalidAccountData);
            }

            if !config.allow_tf_to_dao_wallet(index)
                || *expected_revenue_wallet == Pubkey::default()
            {
                continue;
            }

            WritableAccount::check(revenue_wallet)?;
            WritableAccount::check(revenue_ata)?;

            RevenueWallet::transfer(
                RevenueWalletAccounts {
                    payer_ata: self.accounts.payer_ata,
                    destination_ata: revenue_ata,
                    wallet: revenue_wallet,
                    payer: self.accounts.payer,
                    mint: self.accounts.token_mint,
                    token_program: self.accounts.token_program,
                    associated_token_program: self.accounts.associated_token_program,
                    system_program: self.accounts.system_program,
                },
                RevenueWalletArgs {
                    amount,
                    decimals: config.mint_decimals,
                },
            )?;
        }

        Ok(())
    }

    fn pay_protocol_fee(&self, config: &ConfigV1) -> ProgramResult {
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

    fn mint_nft(self, config: &mut ConfigV1, user_minted: &mut UserMintedV1) -> ProgramResult {
        MplCoreProgram::create(
            CreateMplCoreAssetAccounts {
                payer: self.accounts.payer,
                asset: self.accounts.nft_asset,
                collection: self.accounts.nft_collection,
                authority: Some(self.accounts.nft_authority),
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            CreateMplCoreAssetArgs {
                name: self.instruction_data.nft_name,
                uri: self.instruction_data.nft_uri,
            },
            &[&[NftAuthorityV1::SEED, &[self.nft_authority_bump]]],
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
                ConfigV1::SEED,
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        let (_, nft_authority_bump) =
            Pda::validate(accounts.nft_authority, &[NftAuthorityV1::SEED], program_id)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
            nft_authority_bump,
        })
    }
}

impl<'a, 'info> ProcessInstruction for MintVipV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut config_data = self.accounts.config_pda.try_borrow_mut_data()?;
        let config = ConfigV1::load_mut(config_data.as_mut())?;

        self.init_user_mint_if_needed()?;

        let mut user_minted_data = self.accounts.user_minted_pda.try_borrow_mut_data()?;
        let user_minted = UserMintedV1::load_mut(user_minted_data.as_mut())?;
        if user_minted.has_reached_vip_limit(config) {
            msg!("VIP user has minted their allowed supply");
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

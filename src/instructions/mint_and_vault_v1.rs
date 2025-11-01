use borsh::{BorshDeserialize, BorshSerialize};
use mpl_core::instructions::CreateV1CpiBuilder;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{Config, MintedUser, NftAuthority, Vault},
    utils::{
        AccountCheck, AccountUninitializedCheck, AssociatedTokenAccount,
        AssociatedTokenAccountCheck, AssociatedTokenProgram, ConfigAccount, MintAccount,
        MplCoreAccount, MplCoreAsset, Pda, ProcessInstruction, SignerAccount, SystemAccount,
        SystemProgram, TokenProgram, TransferArgs, WritableAccount,
    },
};

#[derive(Debug)]
pub struct MintAndVaultV1Accounts<'a, 'info> {
    /// User paying the mint price in 'token_mint' and solana.
    /// Must be signer and owner of `payer_ata`.
    pub payer: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "config"]` — stores global config.
    /// Must be readable, owned by program.
    pub config_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "vault"]` — stores `Vault` state.
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
    pub minted_user_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "nft_authority"]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// NFT asset (MPL Core) — the NFT being minted.
    /// Must be uninitialized, owned by `mpl_core`.
    pub nft_asset: &'a AccountInfo<'info>,

    /// User's NFT token account — receives the minted NFT.
    /// Must be writable, owned by `token_program`.
    pub nft_token_account: &'a AccountInfo<'info>,

    /// Token mint — the token being escrowed (e.g. ZDLT.
    /// Must match `config_pda.data.mint`, owned by `token_program`.
    pub token_mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_mint.owner`.
    pub token_program: &'a AccountInfo<'info>,

    /// Protocol wallet — receives the configurable SOL protocol fee.
    /// Must writable, not zero address, owned by system_program.
    pub protocol_wallet: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for MintAndVaultV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [
            payer,
            config_pda,
            vault_pda,
            vault_ata,
            payer_ata,
            minted_user_pda,
            nft_authority,
            nft_asset,
            nft_token_account,
            token_mint,
            token_program,
            protocol_wallet,
            system_program,
            mpl_core,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;
        WritableAccount::check(config_pda)?;
        ConfigAccount::check(config_pda)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        AssociatedTokenAccount::check(vault_ata, vault_pda.key, token_mint.key, token_program.key)?;
        WritableAccount::check(payer_ata)?;
        AssociatedTokenAccount::check(payer_ata, payer.key, token_mint.key, token_program.key)?;
        WritableAccount::check(minted_user_pda)?;
        WritableAccount::check(nft_asset)?;
        MplCoreAsset::check(nft_asset)?;
        MplCoreAsset::check_uninitialized(nft_asset)?;
        WritableAccount::check_uninitialized(nft_asset)?;
        WritableAccount::check(nft_token_account)?;
        AssociatedTokenAccount::check(
            nft_token_account,
            payer.key,
            nft_asset.key,
            token_program.key,
        )?;
        MintAccount::check(token_mint)?;
        WritableAccount::check(protocol_wallet)?;
        SystemAccount::check(system_program)?;
        MplCoreAccount::check(mpl_core)?;

        Ok(Self {
            payer,
            payer_ata,
            vault_ata,
            config_pda,
            vault_pda,
            minted_user_pda,
            nft_authority,
            nft_asset,
            nft_token_account,
            token_mint,
            token_program,
            protocol_wallet,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MintAndVaultV1InstructionData {
    pub name: String,
    pub uri: String,
}

#[derive(Debug)]
pub struct MintAndVaultV1<'a, 'info> {
    pub accounts: MintAndVaultV1Accounts<'a, 'info>,
    pub instruction_data: MintAndVaultV1InstructionData,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info> MintAndVaultV1<'a, 'info> {
    fn check_mint_eligibility(&self, config: &Config) -> ProgramResult {
        if config.supply_minted >= config.max_supply {
            msg!("All nft are minted");
            return Err(ProgramError::Custom(0));
        }

        if config.supply_minted <= config.released {
            msg!("Sold out");
            return Err(ProgramError::Custom(1));
        }

        Ok(())
    }

    fn init_minted_user(&self) -> Result<(), ProgramError> {
        let payer = self.accounts.payer;
        let minted_user_pda = self.accounts.minted_user_pda;
        let system_program = self.accounts.system_program;

        Pda::new(
            payer,
            minted_user_pda,
            system_program,
            &[MintedUser::SEED, payer.key.as_ref()],
            MintedUser::LEN,
            self.program_id,
            self.program_id,
        )?
        .init_if_needed()?;

        let minted = MintedUser::new(*payer.key, false);
        let minted_user_data = &mut minted_user_pda.data.borrow_mut()[..MintedUser::LEN];
        MintedUser::init(minted_user_data, &minted)?;

        Ok(())
    }

    fn transfer_to_vault(
        &self,
        config: &mut Config,
        minted_user: &mut MintedUser,
    ) -> ProgramResult {
        let payer = self.accounts.payer;
        let payer_ata = self.accounts.payer_ata;
        let vault_ata = self.accounts.vault_ata;
        let vault_pda = self.accounts.vault_pda;
        let nft_mint = self.accounts.nft_asset;
        let token_mint = self.accounts.token_mint;
        let token_program = self.accounts.token_program;
        let system_program = self.accounts.system_program;

        let price = config.price;

        if vault_ata.lamports() == 0 {
            AssociatedTokenProgram::init(
                payer,
                vault_pda,
                token_mint,
                token_program,
                system_program,
                vault_ata,
            )?;
        }

        TokenProgram::transfer(TransferArgs {
            source: payer_ata,
            destination: vault_ata,
            authority: payer,
            mint: token_mint,
            token_program,
            signer_pubkeys: &[],
            amount: price,
            decimals: config.mint_decimals,
        })?;

        let vault_bump = Pda::new(
            payer,
            vault_pda,
            system_program,
            &[Vault::SEED, payer.key.as_ref()],
            Vault::LEN,
            self.program_id,
            self.program_id,
        )?
        .init_if_needed()?;

        let vault = Vault::new(*payer.key, *nft_mint.key, price, false, [vault_bump]);
        let vault_data = &mut vault_pda.data.borrow_mut()[..Vault::LEN];
        Vault::init(vault_data, &vault)?;

        minted_user.set_minted(true);
        config.increment_minted()?;

        Ok(())
    }

    fn pay_protocol_fee(&self, config: &Config) -> ProgramResult {
        if config.protocol_fee_lamports == 0 {
            return Ok(());
        }

        let payer = self.accounts.payer;
        let protocol_wallet = self.accounts.protocol_wallet;
        let system_program = self.accounts.system_program;

        SystemProgram::transfer(
            payer,
            protocol_wallet,
            system_program,
            config.protocol_fee_lamports,
        )
    }

    fn mint_nft(self) -> ProgramResult {
        let payer = self.accounts.payer;
        let nft_authority = self.accounts.nft_authority;
        let nft_asset = self.accounts.nft_asset;
        let nft_token_account = self.accounts.nft_token_account;
        let system_program = self.accounts.system_program;
        let mpl_core = self.accounts.mpl_core;

        CreateV1CpiBuilder::new(mpl_core)
            .asset(nft_asset)
            .collection(None)
            .authority(Some(nft_authority))
            .payer(payer)
            .owner(Some(nft_token_account))
            .update_authority(Some(nft_authority))
            .system_program(system_program)
            .name(self.instruction_data.name)
            .uri(self.instruction_data.uri)
            .invoke()?;

        Ok(())
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        MintAndVaultV1InstructionData,
        &'a Pubkey,
    )> for MintAndVaultV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            MintAndVaultV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = MintAndVaultV1Accounts::try_from(accounts)?;

        Pda::validate(accounts.config_pda, &[Config::SEED], program_id)?;

        Pda::validate(accounts.nft_authority, &[NftAuthority::SEED], program_id)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
        })
    }
}

impl<'a, 'info> ProcessInstruction for MintAndVaultV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut config_data = self.accounts.config_pda.data.borrow_mut();
        let config = Config::load_mut(&mut config_data)?;

        self.check_mint_eligibility(config)?;
        self.init_minted_user()?;

        // Read minted user
        let mut minted_user_data = self.accounts.minted_user_pda.data.borrow_mut();
        let minted_user = MintedUser::load_mut(&mut minted_user_data)?;
        if minted_user.is_minted() {
            msg!("Already minted");
            return Err(ProgramError::Custom(2));
        }

        self.transfer_to_vault(config, minted_user)?;
        self.pay_protocol_fee(config)?;
        self.mint_nft()?;

        let price = config.price;

        msg!("MintAndVault: minted NFT and escrowed {} tokens", price);

        Ok(())
    }
}

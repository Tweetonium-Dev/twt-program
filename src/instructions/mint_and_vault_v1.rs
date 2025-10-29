use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar
};
use solana_system_interface::instruction as system_instruction;
use spl_token_interface::instruction as token_instruction;

use crate::{
    states::{Config, MintedUser, Vault, MINTED_USER_SEED, VAULT_SEED},
    utils::{sha256_hash, verify_merkle_proof, AccountCheck, ProcessInstruction, SignerAccount, SystemAccount, WritableAccount},
};

#[derive(Debug)]
pub struct MintAndVaultV1Accounts<'a, 'info> {
    pub authority: &'a AccountInfo<'info>,          // config authority (not required signer for mint)
    pub payer: &'a AccountInfo<'info>,              // user wallet paying price
    pub config_pda: &'a AccountInfo<'info>,         // config PDA
    pub vault_pda: &'a AccountInfo<'info>,          // vault data PDA (holds Vault struct)
    pub vault_ata: &'a AccountInfo<'info>,          // token account owned by vault authority (ATA)
    pub payer_ata: &'a AccountInfo<'info>,          // payer's ZDLT token account (ATA)
    pub minted_user: &'a AccountInfo<'info>,        // usermint PDA (per-wallet minted flag)
    pub nft_mint: &'a AccountInfo<'info>,           // NFT mint
    pub nft_token_account: &'a AccountInfo<'info>,  // user's NFT token account (where minted token will be sent)
    pub token_program: &'a AccountInfo<'info>,      // SPL token program
    pub system_program: &'a AccountInfo<'info>,     // System program
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for MintAndVaultV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [
        authority, 
        payer, 
        config_pda, 
        vault_pda, 
        vault_ata, 
        payer_ata, 
        minted_user, 
        nft_mint, 
        nft_token_account, 
        token_program, 
        system_program
        ] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;
        WritableAccount::check(config_pda)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(payer_ata)?;
        WritableAccount::check(minted_user)?;
        WritableAccount::check(nft_mint)?;
        WritableAccount::check(nft_token_account)?;
        SystemAccount::check(system_program)?;

        Ok(Self {
            authority,
            payer,
            payer_ata,
            vault_ata,
            config_pda,
            vault_pda,
            minted_user,
            nft_mint,
            nft_token_account,
            token_program,
            system_program,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MintAndVaultV1InstructionData {
    pub merkle_proof: Vec<[u8; 32]>,
}

#[derive(Debug)]
pub struct MintAndVaultV1<'a, 'info> {
    pub accounts: MintAndVaultV1Accounts<'a, 'info>,
    pub instruction_data: MintAndVaultV1InstructionData,
    pub program_id: &'a Pubkey,
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

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
        })
    }
}

impl<'a, 'info> ProcessInstruction for MintAndVaultV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let payer = self.accounts.payer;
        let payer_ata = self.accounts.payer_ata;
        let vault_ata = self.accounts.vault_ata;
        let config_pda = self.accounts.config_pda;
        let vault_pda = self.accounts.vault_pda;
        let minted_user = self.accounts.minted_user;
        let nft_mint = self.accounts.nft_mint;
        let token_program = self.accounts.token_program;
        let system_program = self.accounts.system_program;
        let authority = self.accounts.authority;

        // Check mint eligibility from config
        let mut cfg = Config::load(&config_pda.data.borrow())?;

        if cfg.authority != *authority.key {
            msg!("Authority account mismatch with config");
            return Err(ProgramError::InvalidAccountData);
        }

        if cfg.supply_minted >= cfg.max_supply {
            msg!("Sold out");
            return Err(ProgramError::Custom(0));
        }

        if cfg.supply_minted < cfg.released {
            let leaf = sha256_hash(&payer.key.to_bytes());
            if !verify_merkle_proof(leaf, &self.instruction_data.merkle_proof, cfg.merkle_root.to_bytes()) {
                msg!("Not whitelisted");
                return Err(ProgramError::Custom(1));
            }
        }

        // Minted user PDA
        let (expected_minted_user_pda, minted_user_bump) = Pubkey::find_program_address(
            &[MINTED_USER_SEED, payer.key.as_ref()], 
            self.program_id
        );
        if expected_minted_user_pda != *minted_user.key {
            msg!("MintedUser PDA mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        // Create minted user PDA if needed
        if minted_user.lamports() == 0 ||minted_user.data_is_empty() {
            let rent = Rent::get()?;
            let space = MintedUser::LEN;
            let lamports = rent.minimum_balance(space);

            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    minted_user.key,
                    lamports,
                    space as u64,
                    self.program_id,
                ),
                &[payer.clone(), minted_user.clone(), system_program.clone()],
                &[&[MINTED_USER_SEED, payer.key.as_ref(), &[minted_user_bump]]],
            )?;

            let mu = MintedUser {
                owner: *payer.key,
                minted: false,
            };
            mu.serialize(&mut &mut minted_user.data.borrow_mut()[..])?;
        }

        // Read minted user
        let mut usermint = MintedUser::load(&minted_user.data.borrow())?;
        if usermint.minted {
            msg!("Already minted");
            return Err(ProgramError::Custom(2));
        }

        // Transfer token (e.g: ZDLT) from payer -> vault
        let price = cfg.price;
        let transfer_to_vault_ix = token_instruction::transfer(
            token_program.key,
            payer_ata.key,
            vault_ata.key,
            payer.key,
            &[],
           price 
        )?;
        invoke(
            &transfer_to_vault_ix,
            &[
                payer_ata.clone(),
                vault_ata.clone(),
                payer.clone(),
                token_program.clone(),
            ]
        )?;

        // Vault PDA
        let (expected_vault_pda, vault_bump) = Pubkey::find_program_address(
            &[VAULT_SEED, payer.key.as_ref()], 
            self.program_id
        );
        if expected_vault_pda != *vault_pda.key {
            msg!("Vault PDA mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        // Create vault PDA if needed
        if vault_pda.lamports() == 0 || vault_pda.data_is_empty() {
            let rent = Rent::get()?;
            let space = Vault::LEN;
            let lamports = rent.minimum_balance(space);

            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    vault_pda.key,
                    lamports,
                    space as u64,
                    self.program_id,
                ),
                &[payer.clone(), vault_pda.clone(), system_program.clone()],
                &[&[VAULT_SEED, config_pda.key.as_ref(), &[vault_bump]]],
            )?;
        }

        // Write vault record
        let v = Vault {
            owner: *payer.key,
            nft: *nft_mint.key,
            amount: price,
            is_unlocked: false,
            bump: [vault_bump],
        };
        v.serialize(&mut &mut vault_pda.data.borrow_mut()[..])?;

        // Mark minted user as minted
        usermint.minted = true;
        usermint.serialize(&mut &mut minted_user.data.borrow_mut()[..])?;

        // Increment supply minted
        cfg.supply_minted = cfg.supply_minted.checked_add(1).ok_or(ProgramError::Custom(3))?;
        cfg.serialize(&mut &mut config_pda.data.borrow_mut()[..])?;

        // TODO: Mint NFT with mpl-core

        msg!("MintAndVault: minted NFT and escrowed {} tokens", price);
        Ok(())
    }
}

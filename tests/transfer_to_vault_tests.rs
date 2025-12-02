use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};
use tweetonium::{
    instructions::TransferToVaultV1InstructionData,
    process_instruction,
    states::VaultV1,
    utils::{
        ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, mock_base_asset, mock_mint, mock_mint_2022, mock_token_account, mock_token_account_2022, noop_processor
    },
};

#[tokio::test]
async fn test_transfer_to_vault() {
    let program_id = tweetonium::ID;
    let token_program_id = TOKEN_PROGRAM_ID;
    let associated_token_program_id = ASSOCIATED_TOKEN_PROGRAM_ID;
    let system_program_id = solana_program::system_program::id();
    let mpl_core_id = mpl_core::ID;

    let mut program_test = ProgramTest::default();

    // add the tested program and CPI programs
    program_test.add_program("tweetonium", program_id, processor!(process_instruction));
    program_test.add_program("token", token_program_id, processor!(noop_processor));
    program_test.add_program(
        "associated_token",
        associated_token_program_id,
        processor!(noop_processor),
    );

    // --- signers / keys ---
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();

    let nft_collection = Pubkey::new_unique();
    let nft_asset = Pubkey::new_unique();

    let project_token_mint = Pubkey::new_unique();
    let new_token_mint = Pubkey::new_unique();

    // PDAs
    let (payer_ata, _) = Pubkey::find_program_address(
        &[
            payer_pubkey.as_ref(),
            token_program_id.as_ref(),
            new_token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[
            VaultV1::SEED,
            nft_asset.as_ref(),
            nft_collection.as_ref(),
            project_token_mint.as_ref(),
        ],
        &program_id,
    );

    let (new_vault_ata, _) = Pubkey::find_program_address(
        &[
            vault_pda.as_ref(),
            token_program_id.as_ref(),
            new_token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let vault = VaultV1 {
        nft: nft_asset,
        amount: 1_000_000,
        is_unlocked: 0,
        bump: [vault_bump],
    };

    let lamports = 1_000_000_000;

    program_test.add_account(
        payer_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        payer_ata,
        Account {
            lamports,
            data: mock_token_account(&project_token_mint, &payer_pubkey, 1_000_000),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        vault_pda,
        Account {
            lamports,
            data: vault.to_bytes(),
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        nft_collection,
        Account {
            lamports,
            data: vec![],
            owner: mpl_core_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        nft_asset,
        Account {
            lamports,
            data: mock_base_asset(payer_pubkey, "Test NFT", "https://example.com"),
            owner: mpl_core_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        project_token_mint,
        Account {
            lamports,
            data: mock_mint(6, payer_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        new_token_mint,
        Account {
            lamports,
            data: mock_mint(6, payer_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = TransferToVaultV1InstructionData {
        amount: 1_000_000,
    };

    let mut data = vec![11u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer_pubkey, true),
            AccountMeta::new(payer_ata, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(new_vault_ata, false),
            AccountMeta::new(nft_collection, false),
            AccountMeta::new(nft_asset, false),
            AccountMeta::new_readonly(project_token_mint, false),
            AccountMeta::new_readonly(new_token_mint, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(associated_token_program_id, false),
            AccountMeta::new_readonly(system_program_id, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer_pubkey),
        &[&payer],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "TransferToVaultV1 failed: {:?}", result.err());
}

#[tokio::test]
async fn test_transfer_to_vault_2022() {
    let program_id = tweetonium::ID;
    let token_program_id = TOKEN_2022_PROGRAM_ID;
    let associated_token_program_id = ASSOCIATED_TOKEN_PROGRAM_ID;
    let system_program_id = solana_program::system_program::id();
    let mpl_core_id = mpl_core::ID;

    let mut program_test = ProgramTest::default();

    // add the tested program and CPI programs
    program_test.add_program("tweetonium", program_id, processor!(process_instruction));
    program_test.add_program("token", token_program_id, processor!(noop_processor));
    program_test.add_program(
        "associated_token",
        associated_token_program_id,
        processor!(noop_processor),
    );

    // --- signers / keys ---
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();

    let nft_collection = Pubkey::new_unique();
    let nft_asset = Pubkey::new_unique();

    let project_token_mint = Pubkey::new_unique();
    let new_token_mint = Pubkey::new_unique();

    // PDAs
    let (payer_ata, _) = Pubkey::find_program_address(
        &[
            payer_pubkey.as_ref(),
            token_program_id.as_ref(),
            new_token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[
            VaultV1::SEED,
            nft_asset.as_ref(),
            nft_collection.as_ref(),
            project_token_mint.as_ref(),
        ],
        &program_id,
    );

    let (new_vault_ata, _) = Pubkey::find_program_address(
        &[
            vault_pda.as_ref(),
            token_program_id.as_ref(),
            new_token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let vault = VaultV1 {
        nft: nft_asset,
        amount: 1_000_000,
        is_unlocked: 0,
        bump: [vault_bump],
    };

    let lamports = 1_000_000_000;

    program_test.add_account(
        payer_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        payer_ata,
        Account {
            lamports,
            data: mock_token_account_2022(&project_token_mint, &payer_pubkey, 1_000_000),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        vault_pda,
        Account {
            lamports,
            data: vault.to_bytes(),
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        nft_collection,
        Account {
            lamports,
            data: vec![],
            owner: mpl_core_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        nft_asset,
        Account {
            lamports,
            data: mock_base_asset(payer_pubkey, "Test NFT", "https://example.com"),
            owner: mpl_core_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        project_token_mint,
        Account {
            lamports,
            data: mock_mint_2022(6, payer_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        new_token_mint,
        Account {
            lamports,
            data: mock_mint_2022(6, payer_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = TransferToVaultV1InstructionData {
        amount: 1_000_000,
    };

    let mut data = vec![11u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer_pubkey, true),
            AccountMeta::new(payer_ata, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(new_vault_ata, false),
            AccountMeta::new(nft_collection, false),
            AccountMeta::new(nft_asset, false),
            AccountMeta::new_readonly(project_token_mint, false),
            AccountMeta::new_readonly(new_token_mint, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(associated_token_program_id, false),
            AccountMeta::new_readonly(system_program_id, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer_pubkey),
        &[&payer],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "TransferToVaultV1 failed: {:?}", result.err());
}

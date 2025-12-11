use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};
use tweetonium::{
    instructions::MintUserV1InstructionData,
    process_instruction,
    states::{ProjectV1, NftAuthorityV1, UserMintedV1, VaultV1, VestingMode},
    utils::{
        mock_mint, mock_mint_2022, mock_token_account, mock_token_account_2022, noop_processor,
        ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
};

#[tokio::test]
async fn test_mint_user() {
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
    program_test.add_program("mpl_core", mpl_core_id, processor!(noop_processor));

    // --- signers / keys ---
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();

    let nft_collection = Pubkey::new_unique();
    let nft_asset = Keypair::new();
    let nft_asset_pubkey = nft_asset.pubkey();

    let token_mint = Pubkey::new_unique();

    let revenue_wallet_0 = Keypair::new();
    let revenue_wallet_0_pubkey = revenue_wallet_0.pubkey();

    let revenue_wallet_1 = Keypair::new();
    let revenue_wallet_1_pubkey = revenue_wallet_1.pubkey();

    let protocol_wallet = Pubkey::new_unique();

    // PDAs
    let (nft_authority, _) = Pubkey::find_program_address(&[NftAuthorityV1::SEED], &program_id);

    let (payer_ata, _) = Pubkey::find_program_address(
        &[
            payer_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (project_pda, _) = Pubkey::find_program_address(
        &[ProjectV1::SEED, nft_collection.as_ref(), token_mint.as_ref()],
        &program_id,
    );

    let (vault_pda, _) = Pubkey::find_program_address(
        &[
            VaultV1::SEED,
            nft_asset_pubkey.as_ref(),
            nft_collection.as_ref(),
            token_mint.as_ref(),
        ],
        &program_id,
    );

    let (vault_ata, _) = Pubkey::find_program_address(
        &[
            vault_pda.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (user_minted_pda, _) = Pubkey::find_program_address(
        &[
            UserMintedV1::SEED,
            nft_collection.as_ref(),
            token_mint.as_ref(),
            payer_pubkey.as_ref(),
        ],
        &program_id,
    );

    let (revenue_wallet_0_ata, _) = Pubkey::find_program_address(
        &[
            revenue_wallet_0_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (revenue_wallet_1_ata, _) = Pubkey::find_program_address(
        &[
            revenue_wallet_1_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let mut revenue_wallets = [Pubkey::default(); 5];
    revenue_wallets[0] = revenue_wallet_0_pubkey;
    revenue_wallets[1] = revenue_wallet_1_pubkey;

    let mut revenue_shares = [0u64; 5];
    revenue_shares[0] = 5_000_000;
    revenue_shares[1] = 10_000_000;

    let cfg = ProjectV1 {
        admin: payer_pubkey,
        mint: token_mint,
        mint_decimals: 6,
        max_supply: 10_000,
        released: 5_000,
        max_mint_per_user: 5,
        max_mint_per_vip_user: 10,
        admin_minted: 0,
        user_minted: 0,
        vesting_mode: VestingMode::None,
        vesting_unlock_ts: 0,
        mint_nft_fee_lamports: 0,
        update_nft_fee_lamports: 0,
        mint_price_total: 30_000_000,
        escrow_amount: 15_000_000,
        num_revenue_wallets: 0,
        revenue_wallets,
        revenue_shares,
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
            data: mock_token_account(&token_mint, &payer_pubkey, 0),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        project_pda,
        Account {
            lamports,
            data: cfg.to_bytes(),
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        nft_authority,
        Account {
            lamports,
            data: vec![],
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
        token_mint,
        Account {
            lamports,
            data: mock_mint(6, payer_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        revenue_wallet_0_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        revenue_wallet_1_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        protocol_wallet,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = MintUserV1InstructionData {
        nft_name: "Test NFT".to_string(),
        nft_uri: "https://example.com/nft.json".to_string(),
    };

    let mut data = vec![3u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer_pubkey, true),
            AccountMeta::new(payer_ata, false),
            AccountMeta::new(project_pda, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(user_minted_pda, false),
            AccountMeta::new_readonly(nft_authority, false),
            AccountMeta::new(nft_collection, false),
            AccountMeta::new(nft_asset_pubkey, true),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new(revenue_wallet_0_pubkey, false),
            AccountMeta::new(revenue_wallet_0_ata, false),
            AccountMeta::new(revenue_wallet_1_pubkey, false),
            AccountMeta::new(revenue_wallet_1_ata, false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(protocol_wallet, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(associated_token_program_id, false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(mpl_core_id, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer_pubkey),
        &[&payer, &nft_asset],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "MintUserV1 failed: {:?}", result.err());
}

#[tokio::test]
async fn test_mint_user_2022() {
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
    program_test.add_program("mpl_core", mpl_core_id, processor!(noop_processor));

    // --- signers / keys ---
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();

    let nft_collection = Pubkey::new_unique();
    let nft_asset = Keypair::new();
    let nft_asset_pubkey = nft_asset.pubkey();

    let token_mint = Pubkey::new_unique();

    let revenue_wallet_0 = Keypair::new();
    let revenue_wallet_0_pubkey = revenue_wallet_0.pubkey();

    let revenue_wallet_1 = Keypair::new();
    let revenue_wallet_1_pubkey = revenue_wallet_1.pubkey();

    let protocol_wallet = Pubkey::new_unique();

    // PDAs
    let (nft_authority, _) = Pubkey::find_program_address(&[NftAuthorityV1::SEED], &program_id);

    let (payer_ata, _) = Pubkey::find_program_address(
        &[
            payer_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (project_pda, _) = Pubkey::find_program_address(
        &[ProjectV1::SEED, nft_collection.as_ref(), token_mint.as_ref()],
        &program_id,
    );

    let (vault_pda, _) = Pubkey::find_program_address(
        &[
            VaultV1::SEED,
            nft_asset_pubkey.as_ref(),
            nft_collection.as_ref(),
            token_mint.as_ref(),
        ],
        &program_id,
    );

    let (vault_ata, _) = Pubkey::find_program_address(
        &[
            vault_pda.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (user_minted_pda, _) = Pubkey::find_program_address(
        &[
            UserMintedV1::SEED,
            nft_collection.as_ref(),
            token_mint.as_ref(),
            payer_pubkey.as_ref(),
        ],
        &program_id,
    );

    let (revenue_wallet_0_ata, _) = Pubkey::find_program_address(
        &[
            revenue_wallet_0_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let (revenue_wallet_1_ata, _) = Pubkey::find_program_address(
        &[
            revenue_wallet_1_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    let mut revenue_wallets = [Pubkey::default(); 5];
    revenue_wallets[0] = revenue_wallet_0_pubkey;
    revenue_wallets[1] = revenue_wallet_1_pubkey;

    let mut revenue_shares = [0u64; 5];
    revenue_shares[0] = 5_000_000;
    revenue_shares[1] = 10_000_000;

    let cfg = ProjectV1 {
        admin: payer_pubkey,
        mint: token_mint,
        mint_decimals: 6,
        max_supply: 10_000,
        released: 5_000,
        max_mint_per_user: 5,
        max_mint_per_vip_user: 10,
        admin_minted: 0,
        user_minted: 0,
        vesting_mode: VestingMode::None,
        vesting_unlock_ts: 0,
        mint_nft_fee_lamports: 0,
        update_nft_fee_lamports: 0,
        mint_price_total: 30_000_000,
        escrow_amount: 15_000_000,
        num_revenue_wallets: 0,
        revenue_wallets,
        revenue_shares,
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
            data: mock_token_account_2022(&token_mint, &payer_pubkey, 0),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        project_pda,
        Account {
            lamports,
            data: cfg.to_bytes(),
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        nft_authority,
        Account {
            lamports,
            data: vec![],
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
        token_mint,
        Account {
            lamports,
            data: mock_mint_2022(6, payer_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        revenue_wallet_0_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        revenue_wallet_1_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        protocol_wallet,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = MintUserV1InstructionData {
        nft_name: "Test NFT".to_string(),
        nft_uri: "https://example.com/nft.json".to_string(),
    };

    let mut data = vec![3u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer_pubkey, true),
            AccountMeta::new(payer_ata, false),
            AccountMeta::new(project_pda, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(user_minted_pda, false),
            AccountMeta::new_readonly(nft_authority, false),
            AccountMeta::new(nft_collection, false),
            AccountMeta::new(nft_asset_pubkey, true),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new(revenue_wallet_0_pubkey, false),
            AccountMeta::new(revenue_wallet_0_ata, false),
            AccountMeta::new(revenue_wallet_1_pubkey, false),
            AccountMeta::new(revenue_wallet_1_ata, false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(Pubkey::default(), false),
            AccountMeta::new(protocol_wallet, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(associated_token_program_id, false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(mpl_core_id, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer_pubkey),
        &[&payer, &nft_asset],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "MintUserV1 failed: {:?}", result.err());
}

use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};
use tweetonium::{
    instructions::UpdateConfigV1InstructionData,
    process_instruction,
    states::{ConfigV1, NftAuthorityV1, VestingMode},
    utils::{mock_mint, mock_mint_2022, noop_processor, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
};

#[tokio::test]
async fn test_update_config() {
    let program_id = tweetonium::ID;
    let token_program_id = TOKEN_PROGRAM_ID;
    let system_program_id = solana_program::system_program::id();
    let mpl_core_id = mpl_core::ID;

    let mut program_test = ProgramTest::default();

    // add the tested program and CPI programs
    program_test.add_program("tweetonium", program_id, processor!(process_instruction));
    program_test.add_program("mpl_core", mpl_core_id, processor!(noop_processor));

    // --- signers / keys ---
    let admin = Keypair::new();
    let admin_pubkey = admin.pubkey();

    let nft_collection = Pubkey::new_unique();

    let token_mint = Pubkey::new_unique();

    // PDAs
    let (nft_authority, _) = Pubkey::find_program_address(&[NftAuthorityV1::SEED], &program_id);

    let (config_pda, _) = Pubkey::find_program_address(
        &[ConfigV1::SEED, nft_collection.as_ref(), token_mint.as_ref()],
        &program_id,
    );

    let cfg = ConfigV1 {
        admin: admin_pubkey,
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
        revenue_wallets: [Pubkey::default(); 5],
        revenue_shares: [0u64; 5],
    };

    let lamports = 1_000_000_000;

    program_test.add_account(
        admin_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        config_pda,
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
            data: mock_mint(6, admin_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = UpdateConfigV1InstructionData {
        max_supply: 10_000,
        released: 0,
        max_mint_per_user: 5,
        max_mint_per_vip_user: 10,
        vesting_mode: VestingMode::None,
        vesting_unlock_ts: 0,
        mint_nft_fee_lamports: 10_000,
        update_nft_fee_lamports: 5_000,
        mint_price_total: 30_000_000,
        escrow_amount: 15_000_000,
        num_revenue_wallets: 2,
        revenue_wallets: [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ],
        revenue_shares: [5_000_000, 10_000_000, 0, 0, 0],
        num_royalty_recipients: 1,
        royalty_recipients: [
            Pubkey::new_unique(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ],
        royalty_shares_bps: [500, 0, 0, 0, 0],
        collection_name: "Test Collection".to_string(),
        collection_uri: "https://example.com/collection.json".to_string(),
    };

    let mut data = vec![1u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pubkey, true),
            AccountMeta::new(config_pda, false),
            AccountMeta::new_readonly(nft_authority, false),
            AccountMeta::new(nft_collection, false),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(mpl_core_id, false),
        ],
        data,
    };

    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&admin_pubkey), &[&admin], recent_blockhash);

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "UpdateConfigV1 failed: {:?}", result.err());
}

#[tokio::test]
async fn test_update_config_2022() {
    let program_id = tweetonium::ID;
    let token_program_id = TOKEN_2022_PROGRAM_ID;
    let system_program_id = solana_program::system_program::id();
    let mpl_core_id = mpl_core::ID;

    let mut program_test = ProgramTest::default();

    // add the tested program and CPI programs
    program_test.add_program("tweetonium", program_id, processor!(process_instruction));
    program_test.add_program("mpl_core", mpl_core_id, processor!(noop_processor));

    // --- signers / keys ---
    let admin = Keypair::new();
    let admin_pubkey = admin.pubkey();

    let nft_collection = Pubkey::new_unique();

    let token_mint = Pubkey::new_unique();

    // PDAs
    let (nft_authority, _) = Pubkey::find_program_address(&[NftAuthorityV1::SEED], &program_id);

    let (config_pda, _) = Pubkey::find_program_address(
        &[ConfigV1::SEED, nft_collection.as_ref(), token_mint.as_ref()],
        &program_id,
    );

    let cfg = ConfigV1 {
        admin: admin_pubkey,
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
        revenue_wallets: [Pubkey::default(); 5],
        revenue_shares: [0u64; 5],
    };

    let lamports = 1_000_000_000;

    program_test.add_account(
        admin_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        config_pda,
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
            data: mock_mint_2022(6, admin_pubkey),
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = UpdateConfigV1InstructionData {
        max_supply: 10_000,
        released: 0,
        max_mint_per_user: 5,
        max_mint_per_vip_user: 10,
        vesting_mode: VestingMode::None,
        vesting_unlock_ts: 0,
        mint_nft_fee_lamports: 10_000,
        update_nft_fee_lamports: 5_000,
        mint_price_total: 30_000_000,
        escrow_amount: 15_000_000,
        num_revenue_wallets: 2,
        revenue_wallets: [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ],
        revenue_shares: [5_000_000, 10_000_000, 0, 0, 0],
        num_royalty_recipients: 1,
        royalty_recipients: [
            Pubkey::new_unique(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ],
        royalty_shares_bps: [500, 0, 0, 0, 0],
        collection_name: "Test Collection".to_string(),
        collection_uri: "https://example.com/collection.json".to_string(),
    };

    let mut data = vec![1u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pubkey, true),
            AccountMeta::new(config_pda, false),
            AccountMeta::new_readonly(nft_authority, false),
            AccountMeta::new(nft_collection, false),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(mpl_core_id, false),
        ],
        data,
    };

    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&admin_pubkey), &[&admin], recent_blockhash);

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "UpdateConfigV1 failed: {:?}", result.err());
}

use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};
use tweetonium::{
    process_instruction,
    states::{ConfigV1, VestingMode},
    utils::{mock_mint, TOKEN_PROGRAM_ID},
};

#[tokio::test]
async fn test_force_unlock_vesting_v1() {
    let program_id = tweetonium::ID;
    let token_program_id = TOKEN_PROGRAM_ID;
    let system_program_id = solana_program::system_program::id();
    let mpl_core_id = mpl_core::ID;

    let mut program_test = ProgramTest::default();

    // add the tested program and CPI programs
    program_test.add_program("tweetonium", program_id, processor!(process_instruction));

    // --- signers / keys ---
    let admin = Keypair::new();
    let admin_pubkey = admin.pubkey();

    let token_mint = Pubkey::new_unique();

    let nft_collection = Pubkey::new_unique();

    // PDAs

    let (config_pda, _) = Pubkey::find_program_address(
        &[
            ConfigV1::SEED,
            nft_collection.as_ref(),
            token_mint.as_ref(),
        ],
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
        vesting_mode: VestingMode::TimeStamp,
        vesting_unlock_ts: 0,
        mint_nft_fee_lamports: 0,
        update_nft_fee_lamports: 0,
        mint_price_total: 15_000_000,
        escrow_amount: 15_000_000,
        num_revenue_wallets: 0,
        revenue_wallets: [Pubkey::default(); 5],
        revenue_shares: [0; 5],
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
        token_mint,
        Account {
            lamports,
            data: mock_mint(6, admin_pubkey),
            owner: token_program_id,
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

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pubkey, true),
            AccountMeta::new(config_pda, false),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new(nft_collection, false),
        ],
        data: vec![10u8],
    };

    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&admin_pubkey), &[&admin], recent_blockhash);

    let result = banks_client.process_transaction(tx).await;

    assert!(
        result.is_ok(),
        "ForceUnlockVestingV1 failed: {:?}",
        result.err()
    );
}

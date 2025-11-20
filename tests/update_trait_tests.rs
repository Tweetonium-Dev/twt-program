use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};
use tweetonium::{
    instructions::UpdateTraitV1InstructionData,
    process_instruction,
    states::{TraitAuthorityV1, TraitItemV1},
    utils::noop_processor,
};

#[tokio::test]
async fn test_update_trait() {
    let program_id = tweetonium::ID;
    let system_program_id = solana_program::system_program::id();
    let mpl_core_id = mpl_core::ID;

    let mut program_test = ProgramTest::default();

    // add the tested program and CPI programs
    program_test.add_program("tweetonium", program_id, processor!(process_instruction));
    program_test.add_program("mpl_core", mpl_core_id, processor!(noop_processor));

    // --- signers / keys ---
    let authority = Keypair::new();
    let authority_pubkey = authority.pubkey();

    let trait_collection = Pubkey::new_unique();

    // PDAs
    let (trait_authority, _) = Pubkey::find_program_address(&[TraitAuthorityV1::SEED], &program_id);

    let (trait_pda, _) =
        Pubkey::find_program_address(&[TraitItemV1::SEED, trait_collection.as_ref()], &program_id);

    let trait_item = TraitItemV1 {
        authority: authority_pubkey,
        max_supply: 10_000,
        user_minted: 0,
        mint_fee_lamports: 10_000,
    };

    let lamports = 1_000_000_000;

    program_test.add_account(
        authority_pubkey,
        Account {
            lamports,
            data: vec![],
            owner: system_program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        trait_pda,
        Account {
            lamports,
            data: trait_item.to_bytes(),
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        trait_authority,
        Account {
            lamports,
            data: vec![],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        trait_collection,
        Account {
            lamports,
            data: vec![],
            owner: mpl_core_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _bank_payer, recent_blockhash) = program_test.start().await;

    let ix_data = UpdateTraitV1InstructionData {
        max_supply: 10_000,
        mint_fee_lamports: 10_000,
        trait_name: "Update Trait".to_string(),
        trait_uri: "https://example.com/new-trait.json".to_string(),
        num_royalty_recipients: 1,
        royalty_recipients: [
            Pubkey::new_unique(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ],
        royalty_shares_bps: [500, 0, 0, 0, 0],
    };

    let mut data = vec![6u8];
    data.extend(ix_data.try_to_vec().expect("Failed to serialize ix data"));

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority_pubkey, true),
            AccountMeta::new(trait_pda, false),
            AccountMeta::new_readonly(trait_authority, false),
            AccountMeta::new(trait_collection, false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(mpl_core_id, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority_pubkey),
        &[&authority],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;

    assert!(result.is_ok(), "UpdateTraitV1 failed: {:?}", result.err());
}

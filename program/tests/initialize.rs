#![cfg(feature = "test-bpf")]

mod helpers;

use borsh::BorshDeserialize;
use helpers::*;
use simple_token_pool::{id, state::PoolData};
use solana_program::{hash::Hash, instruction::InstructionError};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::TransactionError, transport::TransportError,
};

async fn create_accounts(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    pool_accounts: &PoolAccounts,
) {
    // Create token (incoming)
    create_mint(
        banks_client,
        &payer,
        &recent_blockhash,
        &pool_accounts.bank_mint,
        &pool_accounts.owner.pubkey(),
    )
    .await
    .unwrap();

    // Pool accounts
    create_pool_accounts(
        banks_client,
        payer,
        recent_blockhash,
        &pool_accounts.pool,
        &pool_accounts.pool_mint,
        &pool_accounts.bank,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn success() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    let pool_accounts = PoolAccounts::new();
    create_accounts(&mut banks_client, &payer, &recent_blockhash, &pool_accounts).await;

    pool_accounts
        .initialize(&mut banks_client, &payer, &recent_blockhash)
        .await
        .unwrap();

    let pool = get_account(&mut banks_client, &pool_accounts.pool.pubkey()).await;
    let pool_data = PoolData::try_from_slice(&pool.data).unwrap();

    assert_eq!(pool.owner, id());
    assert_eq!(pool_data.bank, pool_accounts.bank.pubkey())
}

#[tokio::test]
async fn fail_double_initialize() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    let pool_accounts = PoolAccounts::new();
    create_accounts(&mut banks_client, &payer, &recent_blockhash, &pool_accounts).await;

    pool_accounts
        .initialize(&mut banks_client, &payer, &recent_blockhash)
        .await
        .unwrap();

    let latest_blockhash = banks_client.get_recent_blockhash().await.unwrap();

    let tx_error = pool_accounts
        .initialize(&mut banks_client, &payer, &latest_blockhash)
        .await
        .err()
        .unwrap();

    match tx_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(error, InstructionError::AccountAlreadyInitialized);
        }
        _ => panic!("Wrong error"),
    }
}

#[tokio::test]
async fn fail_with_wrong_mint() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    let pool_accounts = PoolAccounts::new();

    // Create mint without init
    create_mint_without_initialize(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &pool_accounts.bank_mint,
    )
    .await
    .unwrap();

    // Pool accounts
    create_pool_accounts(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &pool_accounts.pool,
        &pool_accounts.pool_mint,
        &pool_accounts.bank,
    )
    .await
    .unwrap();

    let tx_error = pool_accounts
        .initialize(&mut banks_client, &payer, &recent_blockhash)
        .await
        .err()
        .unwrap();

    match tx_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(error, InstructionError::UninitializedAccount);
        }
        _ => panic!("Wrong error"),
    }
}

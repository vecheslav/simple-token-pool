#![cfg(feature = "test-bpf")]

mod helpers;

use helpers::*;

use solana_program::{hash::Hash, instruction::InstructionError, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::TransactionError, transport::TransportError,
};

const SENDER_MINT_AMOUNT: u64 = 10000;

async fn prepare_sender(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    pool_accounts: &PoolAccounts,
) -> (Pubkey, Pubkey) {
    let token_sender = Keypair::new();
    let token_recipient = Keypair::new();

    create_token_account(
        banks_client,
        &payer,
        &recent_blockhash,
        &token_sender,
        &pool_accounts.bank_mint.pubkey(),
        &pool_accounts.sender.pubkey(),
    )
    .await
    .unwrap();

    create_token_account(
        banks_client,
        &payer,
        &recent_blockhash,
        &token_recipient,
        &pool_accounts.pool_mint.pubkey(),
        &pool_accounts.sender.pubkey(),
    )
    .await
    .unwrap();

    mint_tokens(
        banks_client,
        &payer,
        &recent_blockhash,
        &pool_accounts.bank_mint.pubkey(),
        &token_sender.pubkey(),
        &pool_accounts.owner,
        SENDER_MINT_AMOUNT,
    )
    .await
    .unwrap();

    (token_sender.pubkey(), token_recipient.pubkey())
}

async fn setup() -> (BanksClient, Keypair, Hash, PoolAccounts) {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let pool_accounts = PoolAccounts::new();
    create_accounts(&mut banks_client, &payer, &recent_blockhash, &pool_accounts).await;

    pool_accounts
        .initialize(&mut banks_client, &payer, &recent_blockhash)
        .await
        .unwrap();

    (banks_client, payer, recent_blockhash, pool_accounts)
}

#[tokio::test]
async fn success() {
    let (mut banks_client, payer, recent_blockhash, pool_accounts) = setup().await;
    let (token_sender, token_recipient) =
        prepare_sender(&mut banks_client, &payer, &recent_blockhash, &pool_accounts).await;

    let sender_balance = get_token_balance(&mut banks_client, &token_sender).await;
    assert_eq!(sender_balance, SENDER_MINT_AMOUNT);

    swap(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &token_sender,
        &token_recipient,
        &pool_accounts,
        5000,
    )
    .await
    .unwrap();

    let new_sender_balance = get_token_balance(&mut banks_client, &token_sender).await;
    assert_eq!(new_sender_balance, SENDER_MINT_AMOUNT - 5000);
}

#[tokio::test]
async fn fail_with_insufficient_tokens() {
    let (mut banks_client, payer, recent_blockhash, pool_accounts) = setup().await;
    let (token_sender, token_recipient) =
        prepare_sender(&mut banks_client, &payer, &recent_blockhash, &pool_accounts).await;

    let sender_balance = get_token_balance(&mut banks_client, &token_sender).await;
    assert_eq!(sender_balance, SENDER_MINT_AMOUNT);

    let tx_error = swap(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &token_sender,
        &token_recipient,
        &pool_accounts,
        SENDER_MINT_AMOUNT + 1,
    )
    .await
    .err()
    .unwrap();

    match tx_error {
        TransportError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(code),
        )) => {
            assert_eq!(code, spl_token::error::TokenError::InsufficientFunds as u32);
        }
        _ => panic!("Wrong error"),
    }
}

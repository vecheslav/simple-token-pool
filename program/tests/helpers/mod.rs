use simple_token_pool::{find_authority_bump_seed, id, instruction, processor, state::PoolData};
use solana_program::{
    borsh::get_packed_len, hash::Hash, program_pack::Pack, pubkey::Pubkey, system_instruction,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account, signature::Keypair, signer::Signer, transaction::Transaction,
    transport::TransportError,
};
use spl_token as token;

#[derive(Debug)]
pub struct PoolAccounts {
    pub owner: Keypair,
    pub pool: Keypair,
    pub bank_mint: Keypair,
    pub pool_mint: Keypair,
    pub bank: Keypair,
    pub sender: Keypair,
    pub recipient: Keypair,
}

impl PoolAccounts {
    pub fn new() -> Self {
        let owner = Keypair::new();
        let pool = Keypair::new();
        let bank_mint = Keypair::new();
        let pool_mint = Keypair::new();
        let bank = Keypair::new();
        let sender = Keypair::new();
        let recipient = Keypair::new();

        Self {
            owner,
            pool,
            bank_mint,
            pool_mint,
            bank,
            sender,
            recipient,
        }
    }

    pub async fn initialize(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Result<(), TransportError> {
        let (authority, _) =
            find_authority_bump_seed(&simple_token_pool::id(), &self.pool.pubkey());

        let mut tx = Transaction::new_with_payer(
            &[instruction::initialize(
                &simple_token_pool::id(),
                &self.pool.pubkey(),
                &authority,
                &self.bank_mint.pubkey(),
                &self.pool_mint.pubkey(),
                &self.bank.pubkey(),
            )],
            Some(&payer.pubkey()),
        );

        tx.sign(&[payer, &self.pool], *recent_blockhash);
        banks_client.process_transaction(tx).await?;

        Ok(())
    }
}

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "simple_token_pool",
        id(),
        processor!(processor::process_instruction),
    )
}

pub async fn get_account(banks_client: &mut BanksClient, pubkey: &Pubkey) -> Account {
    banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn create_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    mint: &Keypair,
    manager: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(token::state::Mint::LEN);

    let mut tx = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                token::state::Mint::LEN as u64,
                &token::id(),
            ),
            token::instruction::initialize_mint(&token::id(), &mint.pubkey(), &manager, None, 0)
                .unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    tx.sign(&[payer, mint], *recent_blockhash);
    banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn create_mint_without_initialize(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    mint: &Keypair,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(token::state::Mint::LEN);

    let mut tx = Transaction::new_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            mint_rent,
            token::state::Mint::LEN as u64,
            &token::id(),
        )],
        Some(&payer.pubkey()),
    );

    tx.sign(&[payer, mint], *recent_blockhash);
    banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn transfer(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    recipient: &Pubkey,
    amount: u64,
) {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &payer.pubkey(),
            recipient,
            amount,
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();
}

/// Create default spl token account and initialize
pub async fn create_token_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(spl_token::state::Account::LEN);

    let mut tx = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    tx.sign(&[payer, account], *recent_blockhash);
    banks_client.process_transaction(tx).await?;

    Ok(())
}

/// Create pool accounts
pub async fn create_pool_accounts(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    pool: &Keypair,
    pool_mint: &Keypair,
    bank: &Keypair,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();

    let pool_rent = rent.minimum_balance(get_packed_len::<PoolData>());
    let pool_mint_rent = rent.minimum_balance(token::state::Mint::LEN);
    let bank_rent = rent.minimum_balance(token::state::Account::LEN);

    let mut tx = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &pool.pubkey(),
                pool_rent,
                get_packed_len::<PoolData>() as u64,
                &simple_token_pool::id(),
            ),
            // Pool mint account
            system_instruction::create_account(
                &payer.pubkey(),
                &pool_mint.pubkey(),
                pool_mint_rent,
                token::state::Mint::LEN as u64,
                &token::id(),
            ),
            // Account for the bank
            system_instruction::create_account(
                &payer.pubkey(),
                &bank.pubkey(),
                bank_rent,
                token::state::Account::LEN as u64,
                &token::id(),
            ),
        ],
        Some(&payer.pubkey()),
    );

    tx.sign(&[payer, pool, pool_mint, bank], *recent_blockhash);
    banks_client.process_transaction(tx).await?;

    Ok(())
}

use borsh::BorshDeserialize;
use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, SubCommand,
};
use simple_token_pool::{
    find_authority_bump_seed,
    instruction::{initialize, swap},
    state::PoolData,
};
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::{pubkey_of, value_of},
    input_validators::{is_amount, is_pubkey, is_url_or_moniker, is_valid_signer},
    keypair::signer_from_path,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{borsh::get_packed_len, program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::*,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token as token;
use std::{env, process::exit};

#[allow(dead_code)]
struct Config {
    rpc_client: RpcClient,
    verbose: bool,
    owner: Box<dyn Signer>,
    fee_payer: Box<dyn Signer>,
}

type Error = Box<dyn std::error::Error>;
type CommandResult = Result<Option<Transaction>, Error>;

macro_rules! unique_signers {
    ($vec:ident) => {
        $vec.sort_by_key(|l| l.pubkey());
        $vec.dedup();
    };
}

fn check_fee_payer_balance(config: &Config, required_balance: u64) -> Result<(), Error> {
    let balance = config.rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            config.fee_payer.pubkey(),
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
        .into())
    } else {
        Ok(())
    }
}

fn command_create_pool(config: &Config, bank_mint_pubkey: &Pubkey) -> CommandResult {
    let pool = Keypair::new();
    println!("Creating pool {}", pool.pubkey());

    let pool_mint = Keypair::new();
    println!("Creating pool mint {}", pool_mint.pubkey());

    let bank = Keypair::new();
    println!("Creating bank account {}", bank.pubkey());

    let pool_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(get_packed_len::<PoolData>())?;
    let pool_mint_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(token::state::Mint::LEN)?;
    let bank_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(token::state::Account::LEN)?;

    let total_required_balance = pool_balance + pool_mint_balance + bank_balance;

    let (authority, _) = find_authority_bump_seed(&simple_token_pool::id(), &pool.pubkey());

    let mut tx = Transaction::new_with_payer(
        &[
            // Pool account
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &pool.pubkey(),
                pool_balance,
                get_packed_len::<PoolData>() as u64,
                &simple_token_pool::id(),
            ),
            // Pool mint account
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &pool_mint.pubkey(),
                pool_mint_balance,
                token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            // Account for the bank
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &bank.pubkey(),
                bank_balance,
                token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            // Initialize pool account
            initialize(
                &simple_token_pool::id(),
                &pool.pubkey(),
                &authority,
                &spl_token::id(),
                &bank_mint_pubkey,
                &pool_mint.pubkey(),
                &bank.pubkey(),
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        config,
        total_required_balance + fee_calculator.calculate_fee(&tx.message()),
    )?;

    let mut signers = vec![config.fee_payer.as_ref(), &pool, &pool_mint, &bank];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn command_swap(config: &Config, pool_pubkey: &Pubkey, amount_in: u64) -> CommandResult {
    let pool = config.rpc_client.get_account(&pool_pubkey)?;
    let pool_data = PoolData::try_from_slice(&pool.data)?;

    println!("{:?}", pool_data);
    println!("Amount: {}", amount_in);

    let (pool_authority, _) = find_authority_bump_seed(&simple_token_pool::id(), &pool_pubkey);

    let mut tx = Transaction::new_with_payer(
        &[swap(
            &simple_token_pool::id(),
            &pool_pubkey,
            &pool_authority,
            &spl_token::id(),
            &pool_data.pool_mint,
            &pool_data.bank_mint,
            &pool_data.bank,
            &config.owner.pubkey(),
            amount_in,
        )?],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&tx.message()))?;

    let mut signers = vec![config.fee_payer.as_ref(), config.owner.as_ref()];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .short("u")
                .long("url")
                .value_name("URL_OR_MONIKER")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help(
                    "URL for Solana's JSON RPC or moniker (or their first letter): \
                       [mainnet-beta, testnet, devnet, localhost] \
                    Default from the configuration file.",
                ),
        )
        .arg(
            Arg::with_name("owner")
                .long("owner")
                .value_name("KEYPAIR")
                .validator(is_valid_signer)
                .takes_value(true)
                .global(true)
                .help(
                    "Specify the token owner account. \
                     This may be a keypair file, the ASK keyword. \
                     Defaults to the client keypair.",
                ),
        )
        .arg(fee_payer_arg().global(true))
        .subcommand(
            SubCommand::with_name("create-pool")
                .about("Create a new pool")
                .arg(
                    Arg::with_name("bank_mint")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help("Mint for bank."),
                ),
        )
        .subcommand(
            SubCommand::with_name("swap")
                .about("Swap to pool tokens")
                .arg(
                    Arg::with_name("pool")
                        .validator(is_pubkey)
                        .value_name("POOL")
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help("Pool public key."),
                )
                .arg(
                    Arg::with_name("amount_in")
                        .validator(is_amount)
                        .value_name("AMOUNT_IN")
                        .takes_value(true)
                        .required(true)
                        .index(2)
                        .help("Amount of tokens for swap."),
                ),
        )
        .get_matches();

    let mut wallet_manager = None;
    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };
        let json_rpc_url = value_t!(matches, "json_rpc_url", String)
            .unwrap_or_else(|_| cli_config.json_rpc_url.clone());

        let owner = signer_from_path(
            &matches,
            matches
                .value_of("owner")
                .unwrap_or(&cli_config.keypair_path),
            "owner",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });
        let fee_payer = signer_from_path(
            &matches,
            matches
                .value_of("fee_payer")
                .unwrap_or(&cli_config.keypair_path),
            "fee_payer",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });
        let verbose = matches.is_present("verbose");

        Config {
            rpc_client: RpcClient::new_with_commitment(json_rpc_url, CommitmentConfig::confirmed()),
            verbose,
            owner,
            fee_payer,
        }
    };

    solana_logger::setup_with_default("solana=info");

    let _ = match matches.subcommand() {
        ("create-pool", Some(arg_matches)) => {
            let bank_mint = pubkey_of(arg_matches, "bank_mint").unwrap();
            command_create_pool(&config, &bank_mint)
        }
        ("swap", Some(arg_matches)) => {
            let pool = pubkey_of(arg_matches, "pool").unwrap();
            let amount_in = value_of::<u64>(arg_matches, "amount_in").unwrap();
            command_swap(&config, &pool, amount_in)
        }
        _ => unreachable!(),
    }
    .and_then(|tx| {
        if let Some(tx) = tx {
            let signature = config
                .rpc_client
                .send_and_confirm_transaction_with_spinner(&tx)?;
            println!("Signature: {}", signature);
        }
        Ok(())
    })
    .map_err(|err| {
        eprintln!("{}", err);
        exit(1);
    });
}

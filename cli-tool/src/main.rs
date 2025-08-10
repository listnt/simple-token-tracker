use clap::{Parser, Subcommand};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer, write_keypair_file},
    system_program,
    transaction::Transaction,
};
use borsh::{BorshSerialize, BorshDeserialize};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};
use spl_token::{
    instruction::{initialize_mint, mint_to, transfer, AuthorityType},
    state::{Account as SplTokenAccount, Mint as SplMint},
};
use std::str::FromStr;
use anyhow::{Result, anyhow};
use solana_sdk::program_pack::Pack;
use sha2::{Sha256, Digest};

// Replace with your deployed program ID and token mint address
const PROGRAM_ID_STR: &str = "E8jj31VT5EMpWq8mqJVh8rXGUBCet6r31u41SELzSQb9";
const TOKEN_MINT_STR: &str = "8v6waw8VPzwnfjZ3j6mFrsBNXZVb7hZLjmgSYC1KSAK8";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deposit tokens into the program
    Deposit {
        #[arg(short, long)]
        amount: u64,
        #[arg(short, long, value_name = "FILE")]
        keypair: String,
    },
    /// Withdraw tokens from the program
    Withdraw {
        #[arg(short, long)]
        amount: u64,
        #[arg(short, long, value_name = "FILE")]
        keypair: String,
    },
    /// Get the balance of a user's account in the program
    Balance {
        #[arg(short, long, value_name = "FILE")]
        keypair: String,
    },
}

fn anchor_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", name));
    let hash = hasher.finalize();
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}


fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    let program_id = Pubkey::from_str(PROGRAM_ID_STR)?;
    let token_mint = Pubkey::from_str(TOKEN_MINT_STR)?;

    match &cli.command {
        Commands::Deposit { amount, keypair } => {
            let user_keypair = read_keypair_file(keypair).map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;
            deposit_tokens(&rpc_client, &program_id, &token_mint, &user_keypair, *amount)?;
        }
        Commands::Withdraw { amount, keypair } => {
            let user_keypair = read_keypair_file(keypair).map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;
            withdraw_tokens(&rpc_client, &program_id, &token_mint, &user_keypair, *amount)?;
        }
        Commands::Balance { keypair } => {
            let user_keypair = read_keypair_file(keypair).map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;
            get_balance(&rpc_client, &program_id, &user_keypair.pubkey())?;
        }
    }

    Ok(())
}

fn deposit_tokens(
    client: &RpcClient,
    program_id: &Pubkey,
    token_mint: &Pubkey,
    user_keypair: &solana_sdk::signature::Keypair,
    amount: u64,
) -> Result<()> {
    println!("Depositing {} tokens...", amount);
    let user_pubkey = user_keypair.pubkey();
    let (program_authority_pda, _bump) = Pubkey::find_program_address(&[b"authority"], program_id);
    let user_token_account = get_associated_token_address(&user_pubkey, token_mint);
    let program_token_account = get_associated_token_address(&program_authority_pda, token_mint);
    let (user_state_pda, _bump2) = Pubkey::find_program_address(&[b"token_state", user_pubkey.as_ref()], program_id);

    if client.get_account(&program_token_account).is_err() {
        println!("Creating program's associated token account...");
        let create_ata_ix = create_associated_token_account(
            &user_pubkey,             // payer
            &program_authority_pda,   // owner of the new ATA
            token_mint,               // mint
            &spl_token::id(),
        );

        let mut tx = Transaction::new_with_payer(&[create_ata_ix], Some(&user_pubkey));
        let latest_blockhash = client.get_latest_blockhash()?;
        tx.sign(&[user_keypair], latest_blockhash);
        client.send_and_confirm_transaction(&tx)?;
        println!("Program ATA created: {}", program_token_account);
    }
    println!("Depositing ...");

    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("deposit"));
    data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new(user_token_account, false),
        AccountMeta::new(program_token_account, false),
        AccountMeta::new(user_state_pda, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    let instruction = Instruction {
        program_id: *program_id, // Anchor program id (must match declare_id!)
        accounts,
        data: data,
    };

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
    let latest_blockhash = client.get_latest_blockhash()?;
    tx.sign(&[user_keypair], latest_blockhash);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("✅ Deposit successful! Signature: {}", sig);

    Ok(())
}

fn withdraw_tokens(
    client: &RpcClient,
    program_id: &Pubkey,
    token_mint: &Pubkey,
    user_keypair: &solana_sdk::signature::Keypair,
    amount: u64,
) -> Result<()> {
    println!("Withdrawing {} tokens...", amount);

    let user_pubkey = user_keypair.pubkey();
    let user_token_account = get_associated_token_address(&user_pubkey, token_mint);
    let (program_authority_pda, _bump) = Pubkey::find_program_address(&[b"authority"], program_id);
    let program_token_account = get_associated_token_address(&program_authority_pda, token_mint);
    let (user_state_pda, _bump) = Pubkey::find_program_address(&[b"token_state", user_pubkey.as_ref()], program_id);

    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("withdraw"));
    data.extend_from_slice(&amount.to_le_bytes());

    let mut transaction = Transaction::new_with_payer(
        &[
            Instruction {
                program_id: *program_id,
                accounts: vec![
                    AccountMeta::new(user_keypair.pubkey(), true),
                    AccountMeta::new(user_token_account, false),
                    AccountMeta::new(program_token_account, false),
                    AccountMeta::new(program_authority_pda, false),
                    AccountMeta::new(user_state_pda, false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: data,
            }
        ],
        Some(&user_keypair.pubkey()),
    );

    let latest_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[user_keypair], latest_blockhash);

    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("✅ Withdrawal successful! Transaction signature: {}", signature);

    Ok(())
}

fn get_balance(client: &RpcClient, program_id: &Pubkey, user_pubkey: &Pubkey) -> Result<()> {
    let (user_state_pda, _bump) = Pubkey::find_program_address(&[b"token_state", user_pubkey.as_ref()], program_id);

    match client.get_account_data(&user_state_pda) {
        Ok(data) => {
            // Anchor-serialized data starts after an 8-byte discriminator
            let balance: u64 = u64::from_le_bytes(data[8..16].try_into()?);
            println!("Current balance for user {}: {} tokens", user_pubkey, balance);
        }
        Err(_) => {
            println!("No balance found for user {}. Have you made a deposit yet?", user_pubkey);
        }
    }

    Ok(())
}
# Simple Solana Token Tracker

This project is a basic token tracking application on the Solana blockchain. It includes a smart contract (program) written in Rust using the Anchor framework and a command-line interface (CLI) for user interaction.

**Right now deployed in devnet, feel free to roam around**

## Features

- **Deposit:** Users can deposit any SPL token into the program.
- **Withdraw:** Users can withdraw tokens back to their wallet.
- **Balance Tracking:** The program maintains an on-chain account for each user to track their balance.
- **Events:** Emits `DepositEvent` and `WithdrawEvent` for easy transaction history tracking.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli)
- [Anchor CLI](https://book.anchor-lang.com/getting_started/installation.html)

### Setup and Deployment

1.  **Clone the repository:**

2.  **Build and deploy the Solana program to devnet:**
    ```bash
    anchor build
    anchor deploy
    ```
    *Note: Anchor will automatically update the program ID in `lib.rs` and `Anchor.toml`.*

3.  **Create a new SPL token and fund your account:**
    ```bash
    spl-token create-token
    spl-token create-account <MINT_ADDRESS>
    spl-token mint <MINT_ADDRESS> <AMOUNT> <YOUR_ACCOUNT_ADDRESS>
    ```

### CLI Usage

The CLI tool is located in the `cli-tool` directory.

P.S. You probably want to recompile cli tool with your own token.  
Or you can ask [me](https://github.com/listnt) to mint you some


1.  **Build the CLI:**
    ```bash
    cd cli-tool
    cargo build --release
    ```

2.  **Interact with the program:**

    - **Check balance:**
      ```bash
      ./target/release/cli-tool balance --keypair <path-to-your-keypair.json>
      ```

    - **Deposit tokens:**
      ```bash
      ./target/release/cli-tool deposit --amount <amount> --keypair <path-to-your-keypair.json>
      ```

    - **Withdraw tokens:**
      ```bash
      ./target/release/cli-tool withdraw <amount> --keypair <path-to-your-keypair.json>
      ```

## Project Structure

- `programs/token_tracker/src/lib.rs`: The Solana smart contract logic.
- `cli-tool/src/main.rs`: The command-line interface tool.
- `Anchor.toml`: Anchor configuration file.
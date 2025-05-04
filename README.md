# 🌊 Solana Liquidity Pool AMM
A Solana-based Automated Market Maker (AMM) program implemented in Rust, inspired by Raydium, enabling decentralized token swaps, liquidity provision, and pool management on the Solana blockchain.

## 📖 Overview
This project is a Solana program that implements a liquidity pool with AMM functionality. It allows users to:
Initialize a liquidity pool with a token pair (e.g., SOL/USDC).

Add liquidity to the pool and receive liquidity tokens.

Remove liquidity by burning liquidity tokens.

Swap tokens using a constant product formula with a 0.3% fee.

The program is deployed on Solana's devnet and includes a Rust client for testing all instructions.

## ✨ Features
InitializePool: Creates a pool with a specified token pair and vault accounts.

AddLiquidity: Deposits tokens into the pool, minting liquidity tokens.

RemoveLiquidity: Burns liquidity tokens to withdraw proportional tokens.

Swap: Executes token swaps with a constant product formula (x * y = k).

Security: Uses Program Derived Addresses (PDAs) for authority, extensive account validation, and custom error handling.

Testing: Includes a Rust client to test all instructions on devnet with real SPL tokens.

## 🔧 Prerequisites
Rust: cargo, rustc (latest stable).

Solana CLI: solana-cli (version 1.18.9 recommended).

Node.js (optional for additional tooling).

A funded Solana keypair on devnet.

## 🚀 Installation
Clone the Repository:
bash

git clone https://github.com/your-username/solana-liquidity-pool.git
cd solana-liquidity-pool

Install Dependencies:
bash

cargo build

Configure Solana CLI:
bash

solana config set --url https://api.devnet.solana.com

🏗️ Building and Deploying
Build the Program:
bash

cargo build-bpf

Deploy to Devnet:
bash

solana program deploy target/deploy/solana_liquidity_pool.so

Note the program ID (e.g., 2SRp5ENH631KzuRGNXWcdLi59pnvKNNoTm37pMRcBH3Q).

🧪 Testing
Set Up Test Client:
bash

cd solana-liquidity-pool-client
cargo build

Create SPL Tokens:
bash

spl-token create-token
spl-token create-account <Token_Mint_Pubkey>
spl-token mint <Token_Mint_Pubkey> 1000000

Repeat for two tokens and vault accounts.

Run Tests:
bash

cargo run

Tests InitializePool, AddLiquidity, RemoveLiquidity, and Swap instructions.

Project Structure
src/lib.rs: Core AMM program with instruction logic and data structures.

solana-liquidity-pool-client/: Rust client for testing instructions on devnet.

target/deploy/: Compiled .so file for deployment.

Security Considerations
PDA Authority: Ensures only the program can control vault transfers.

Validations: Checks account ownership, writability, and state to prevent unauthorized actions.

No Rug Pull/Honeypot: Code is transparent, with no hidden withdrawal mechanisms.

Future Enhancements
Add slippage protection for swaps.

Implement fee distribution for liquidity providers.

Support program upgrades via BPFLoaderUpgradeable.

Audit for mainnet deployment.

License
MIT License. See LICENSE for details.
Contributing
Contributions are welcome! Please open an issue or submit a pull request for improvements or bug fixes.
